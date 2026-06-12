//! Connection-reuse pool primitive.
//!
//! spec: S-88 / t0355

use crate::url::Url;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

/// Errors that can occur during pool operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    /// The URL scheme is not supported.
    UnsupportedScheme,
    /// The URL host is missing.
    MissingHost,
    /// The maximum number of connections for this host has been reached.
    MaxConnectionsReached,
    /// The connection was not found in the pool.
    ConnectionNotFound,
    /// The connection is already in an idle state.
    ConnectionAlreadyIdle,
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolError::UnsupportedScheme => write!(f, "Unsupported scheme"),
            PoolError::MissingHost => write!(f, "Missing host"),
            PoolError::MaxConnectionsReached => write!(f, "Maximum connections reached for origin"),
            PoolError::ConnectionNotFound => write!(f, "Connection not found in pool"),
            PoolError::ConnectionAlreadyIdle => write!(f, "Connection already idle"),
        }
    }
}

impl std::error::Error for PoolError {}

/// An origin represented by scheme, host, and port.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Origin {
    /// The normalized lowercase scheme.
    pub scheme: String,
    /// The host name.
    pub host: String,
    /// The port number.
    pub port: u16,
}

impl Origin {
    /// Creates an `Origin` from a `Url`.
    pub fn from_url(url: &Url) -> Result<Self, PoolError> {
        let scheme = url.scheme.to_lowercase();
        let host = match &url.host {
            Some(h) if !h.is_empty() => h.clone(),
            _ => return Err(PoolError::MissingHost),
        };
        let port = match url.port {
            Some(p) => p,
            None => match scheme.as_str() {
                "http" | "ws" => 80,
                "https" | "wss" => 443,
                "ftp" => 21,
                _ => return Err(PoolError::UnsupportedScheme),
            },
        };
        Ok(Origin { scheme, host, port })
    }
}

/// A connection slot representation in the pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// Unique identifier for this connection slot.
    pub id: usize,
    /// The origin of this connection.
    pub origin: Origin,
    /// How many times this connection has been reused.
    pub reuse_count: usize,
}

#[derive(Debug)]
struct PooledConnection {
    id: usize,
    origin: Origin,
    in_use: bool,
    reuse_count: usize,
}

/// A host-keyed connection pool for managing connection reuse.
#[derive(Debug)]
pub struct ConnectionPool {
    max_per_host: usize,
    state: Mutex<PoolState>,
}

#[derive(Debug, Default)]
struct PoolState {
    next_id: usize,
    connections: HashMap<Origin, Vec<PooledConnection>>,
}

impl ConnectionPool {
    /// Creates a new `ConnectionPool` with a bounded number of connections per host.
    pub fn new(max_per_host: usize) -> Self {
        let max_per_host = max_per_host.max(1);
        ConnectionPool {
            max_per_host,
            state: Mutex::new(PoolState::default()),
        }
    }

    /// Acquires a connection slot for the given URL.
    ///
    /// If an idle connection slot is available, it is reused.
    /// Otherwise, if the number of connections for the origin is below `max_per_host`,
    /// a new slot is allocated.
    /// If the limit is reached, it returns `Err(PoolError::MaxConnectionsReached)`.
    pub fn acquire(&self, url: &Url) -> Result<Connection, PoolError> {
        let origin = Origin::from_url(url)?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        let mut found_conn = None;
        let mut can_allocate = false;

        if let Some(list) = state.connections.get_mut(&origin) {
            if let Some(conn) = list.iter_mut().find(|c| !c.in_use) {
                conn.in_use = true;
                conn.reuse_count += 1;
                found_conn = Some(Connection {
                    id: conn.id,
                    origin: conn.origin.clone(),
                    reuse_count: conn.reuse_count,
                });
            } else if list.len() < self.max_per_host {
                can_allocate = true;
            } else {
                return Err(PoolError::MaxConnectionsReached);
            }
        } else {
            can_allocate = true;
        }

        if let Some(conn) = found_conn {
            // TODO(spec): bind real transport here
            return Ok(conn);
        }

        if can_allocate {
            let id = state.next_id;
            state.next_id += 1;
            let pooled = PooledConnection {
                id,
                origin: origin.clone(),
                in_use: true,
                reuse_count: 0,
            };
            state
                .connections
                .entry(origin.clone())
                .or_default()
                .push(pooled);
            // TODO(spec): bind real transport here
            Ok(Connection {
                id,
                origin,
                reuse_count: 0,
            })
        } else {
            Err(PoolError::MaxConnectionsReached)
        }
    }

    /// Releases a connection slot back to the pool, marking it idle so it can be reused.
    pub fn release(&self, connection: Connection) -> Result<(), PoolError> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(list) = state.connections.get_mut(&connection.origin)
            && let Some(conn) = list.iter_mut().find(|c| c.id == connection.id)
        {
            if !conn.in_use {
                return Err(PoolError::ConnectionAlreadyIdle);
            }
            conn.in_use = false;
            return Ok(());
        }
        Err(PoolError::ConnectionNotFound)
    }

    /// Returns the number of idle connections currently in the pool for this origin.
    pub fn idle_count(&self, origin: &Origin) -> usize {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .connections
            .get(origin)
            .map(|list| list.iter().filter(|c| !c.in_use).count())
            .unwrap_or(0)
    }

    /// Returns the number of in-use (active) connections currently in the pool for this origin.
    pub fn active_count(&self, origin: &Origin) -> usize {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .connections
            .get(origin)
            .map(|list| list.iter().filter(|c| c.in_use).count())
            .unwrap_or(0)
    }

    /// Returns the total number of connections (idle + active) currently in the pool for this origin.
    pub fn total_count(&self, origin: &Origin) -> usize {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .connections
            .get(origin)
            .map(|list| list.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_from_url() {
        let url = Url::parse("http://example.com/").unwrap();
        let origin = Origin::from_url(&url).unwrap();
        assert_eq!(origin.scheme, "http");
        assert_eq!(origin.host, "example.com");
        assert_eq!(origin.port, 80);

        let url_port = Url::parse("https://example.com:8443/").unwrap();
        let origin_port = Origin::from_url(&url_port).unwrap();
        assert_eq!(origin_port.scheme, "https");
        assert_eq!(origin_port.host, "example.com");
        assert_eq!(origin_port.port, 8443);
    }

    #[test]
    fn test_acquire_and_release_reuse() {
        let pool = ConnectionPool::new(2);
        let url = Url::parse("http://example.com/").unwrap();
        let origin = Origin::from_url(&url).unwrap();

        // 1. First acquire: allocates a new connection slot.
        let conn1 = pool.acquire(&url).unwrap();
        assert_eq!(conn1.origin, origin);
        assert_eq!(conn1.reuse_count, 0);
        assert_eq!(pool.total_count(&origin), 1);
        assert_eq!(pool.active_count(&origin), 1);
        assert_eq!(pool.idle_count(&origin), 0);

        // 2. Release connection back to pool.
        pool.release(conn1.clone()).unwrap();
        assert_eq!(pool.total_count(&origin), 1);
        assert_eq!(pool.active_count(&origin), 0);
        assert_eq!(pool.idle_count(&origin), 1);

        // 3. Acquire again: should reuse the same slot.
        let conn2 = pool.acquire(&url).unwrap();
        assert_eq!(conn2.id, conn1.id);
        assert_eq!(conn2.reuse_count, 1); // Reuse count incremented
        assert_eq!(pool.total_count(&origin), 1);
        assert_eq!(pool.active_count(&origin), 1);
        assert_eq!(pool.idle_count(&origin), 0);
    }

    #[test]
    fn test_different_origins_independent() {
        let pool = ConnectionPool::new(2);
        let url_a = Url::parse("http://a.com/").unwrap();
        let url_b = Url::parse("http://b.com/").unwrap();

        let conn_a = pool.acquire(&url_a).unwrap();
        let conn_b = pool.acquire(&url_b).unwrap();

        assert_ne!(conn_a.origin, conn_b.origin);
        assert_eq!(pool.total_count(&conn_a.origin), 1);
        assert_eq!(pool.total_count(&conn_b.origin), 1);
    }

    #[test]
    fn test_max_per_host_bound() {
        let pool = ConnectionPool::new(2);
        let url = Url::parse("http://example.com/").unwrap();
        let origin = Origin::from_url(&url).unwrap();

        // Acquire up to limit
        let conn1 = pool.acquire(&url).unwrap();
        let conn2 = pool.acquire(&url).unwrap();
        assert_ne!(conn1.id, conn2.id);
        assert_eq!(pool.total_count(&origin), 2);
        assert_eq!(pool.active_count(&origin), 2);

        // Attempting to acquire a third should fail because of limit
        let conn3_res = pool.acquire(&url);
        assert_eq!(conn3_res, Err(PoolError::MaxConnectionsReached));

        // Release one, and try to acquire again - should succeed
        pool.release(conn1).unwrap();
        assert_eq!(pool.active_count(&origin), 1);
        assert_eq!(pool.idle_count(&origin), 1);

        let conn4 = pool.acquire(&url).unwrap();
        assert_eq!(conn4.reuse_count, 1);
        assert_eq!(pool.active_count(&origin), 2);
    }

    #[test]
    fn test_invalid_release_errors() {
        let pool = ConnectionPool::new(2);
        let url = Url::parse("http://example.com/").unwrap();
        let conn = pool.acquire(&url).unwrap();

        // Release it once: should be fine
        pool.release(conn.clone()).unwrap();

        // Releasing it again should fail
        assert_eq!(
            pool.release(conn.clone()),
            Err(PoolError::ConnectionAlreadyIdle)
        );

        // Releasing a non-existent connection should fail
        let mut unknown_conn = conn;
        unknown_conn.id = 999;
        assert_eq!(
            pool.release(unknown_conn),
            Err(PoolError::ConnectionNotFound)
        );
    }
}
