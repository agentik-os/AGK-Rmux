from engine.calibration import Forecast, brier_score

def test_perfect():
    assert brier_score([Forecast(1,1), Forecast(0,0)]) == 0
