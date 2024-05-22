# Calculates the lookup tables used by the NES APU mixer
# to quickly output the resulting sample combination

pulse_table = [0] * 31
tnd_table = [0] * 203

for pulse1 in range(0, 16):
    for pulse2 in range(0, 16):
        n = pulse1 + pulse2

        if n > 0:
            pulse_table[n] = 95.52 / (8128.0 / n + 100.0)

for triangle in range(0, 16):
    for noise in range(0, 16):
        for dmc in range(0, 128):
            n = 3 * triangle + 2 * noise + dmc

            if n > 0:
                tnd_table[n] = 163.67 / (24329.0 / n + 100.0)

assert(max(pulse_table) + max(tnd_table) <= 1, "values must be in [0, 1]")

print("Pulse Table: ", pulse_table)
print("Tnd Table: ", tnd_table)