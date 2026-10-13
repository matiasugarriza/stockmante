//! Reloj inyectable para expiración de sesiones testeable (Principio IV).

use chrono::{DateTime, Utc};

pub trait Reloj: Send + Sync {
    fn ahora(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RelojSistema;

impl Reloj for RelojSistema {
    fn ahora(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Para tests: tiempo controlado.
#[derive(Debug, Clone, Copy)]
pub struct RelojFalso {
    pub ahora: DateTime<Utc>,
}

impl Reloj for RelojFalso {
    fn ahora(&self) -> DateTime<Utc> {
        self.ahora
    }
}
