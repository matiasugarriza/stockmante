import { describe, expect, it } from 'vitest';
import { esRutaPublica, RUTAS_PUBLICAS } from '#lib/utils/rutas';

// T053 (TDD): FR-070 exige que el formulario de registro sea accesible públicamente.
// El guard del $layout sólo debe patear a /login desde rutas NO públicas.
describe('esRutaPublica', () => {
  it('trata como públicas las rutas de autenticación', () => {
    expect(esRutaPublica('/login')).toBe(true);
    expect(esRutaPublica('/registro')).toBe(true);
    expect(esRutaPublica('/verificar-correo')).toBe(true);
  });

  it('ignora trailing slash', () => {
    expect(esRutaPublica('/registro/')).toBe(true);
    expect(esRutaPublica('/verificar-correo/')).toBe(true);
  });

  it('no trata como pública ninguna ruta protegida', () => {
    expect(esRutaPublica('/')).toBe(false);
    expect(esRutaPublica('/caja')).toBe(false);
    expect(esRutaPublica('/movimientos')).toBe(false);
    expect(esRutaPublica('/admin/usuarios')).toBe(false);
  });

  it('no confunde prefijos que comparten string', () => {
    expect(esRutaPublica('/registro-avanzado')).toBe(false);
    expect(esRutaPublica('/loginx')).toBe(false);
  });

  it('lista exactamente las tres rutas públicas', () => {
    expect([...RUTAS_PUBLICAS].sort()).toEqual(['/login', '/registro', '/verificar-correo']);
  });
});