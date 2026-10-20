//! Modelo público de usuario (sin `password_hash`: nunca sale del backend).

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Usuario tal como lo devuelven servicios y handlers.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Usuario {
    pub id: i32,
    pub nombre: String,
    pub email: String,
    pub rol: String,
    pub familia_rol: String,
    pub activo: bool,
    pub password_temporal: bool,
    pub creado_en: DateTime<Utc>,
}

/// Fila interna del login: incluye el hash solo para verificar credenciales.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UsuarioConHash {
    pub id: i32,
    pub nombre: String,
    pub email: String,
    pub password_hash: String,
    pub rol: String,
    pub familia_rol: String,
    pub activo: bool,
    pub password_temporal: bool,
    pub creado_en: DateTime<Utc>,
}

impl UsuarioConHash {
    pub fn into_publica(self) -> Usuario {
        Usuario {
            id: self.id,
            nombre: self.nombre,
            email: self.email,
            rol: self.rol,
            familia_rol: self.familia_rol,
            activo: self.activo,
            password_temporal: self.password_temporal,
            creado_en: self.creado_en,
        }
    }
}

/// Sesión vigente con el usuario dueño y el instante de expiración.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SesionVigente {
    pub id: i32,
    pub nombre: String,
    pub email: String,
    pub rol: String,
    pub familia_rol: String,
    pub activo: bool,
    pub password_temporal: bool,
    pub creado_en: DateTime<Utc>,
    pub expira_en: DateTime<Utc>,
}

impl SesionVigente {
    pub fn into_usuario(self) -> Usuario {
        Usuario {
            id: self.id,
            nombre: self.nombre,
            email: self.email,
            rol: self.rol,
            familia_rol: self.familia_rol,
            activo: self.activo,
            password_temporal: self.password_temporal,
            creado_en: self.creado_en,
        }
    }
}
