"""Compara GraviLens con lenstronomy para una lente puntual.

Uso:  pip install lenstronomy astropy   y luego   python tools/validate_lenstronomy.py
Los valores de GraviLens son los de los tests de src/lensing/tests.rs.
"""
import numpy as np
from astropy import constants as c, units as u
from lenstronomy.LensModel.lens_model import LensModel
from lenstronomy.LensEquationSolver.lens_equation_solver import LensEquationSolver

# Escena: lente de 1 masa solar a 4 kpc, fuente a 8 kpc.
mass, d_l, d_s = 1 * u.M_sun, 4 * u.kpc, 8 * u.kpc
d_ls = d_s - d_l
theta_e = np.sqrt(4 * c.G * mass / c.c**2 * d_ls / (d_l * d_s)).to(u.dimensionless_unscaled)
theta_e_mas = (theta_e * u.rad).to(u.mas).value
print(f"theta_E analítico:     {theta_e_mas:.4f} mas (GraviLens: 1.01)")

# lenstronomy trabaja en unidades arbitrarias: se usa mas directamente.
lens = LensModel(["POINT_MASS"])
kwargs = [{"theta_E": theta_e_mas, "center_x": 0.0, "center_y": 0.0}]
solver = LensEquationSolver(lens)
for beta in (0.0001, 0.7 * theta_e_mas, 2.0):
    x, y = solver.image_position_from_source(beta, 0.0, kwargs, min_distance=0.01, search_window=20)
    mu = np.abs(lens.magnification(x, y, kwargs))
    plus = (beta + np.sqrt(beta**2 + 4 * theta_e_mas**2)) / 2
    minus = (beta - np.sqrt(beta**2 + 4 * theta_e_mas**2)) / 2
    print(f"beta={beta:.4f} mas | lenstronomy x={np.sort(x)} mu={mu.sum():.4f} | GraviLens θ+={plus:.4f} θ−={minus:.4f}")
