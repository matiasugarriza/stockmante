//! Cálculos puros de los reportes (014, R3, R5).

/// Saldo corrido del kardex: cada posición es el saldo después de esa línea.
pub fn acumular_saldos(saldo_inicial: i32, deltas: &[i32]) -> Vec<i32> {
    deltas
        .iter()
        .scan(saldo_inicial, |saldo, delta| {
            *saldo = saldo.saturating_add(*delta);
            Some(*saldo)
        })
        .collect()
}

/// `(promedio, indice)`: promedio = (inicial + final) / 2; índice = salidas / promedio,
/// 0 cuando el promedio es 0.
pub fn calcular_rotacion(unidades_salidas: i32, stock_inicial: i32, stock_final: i32) -> (f64, f64) {
    let promedio = (f64::from(stock_inicial) + f64::from(stock_final)) / 2.0;
    if promedio <= 0.0 {
        return (0.0, 0.0);
    }
    (promedio, f64::from(unidades_salidas) / promedio)
}
