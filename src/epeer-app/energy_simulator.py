import random

class EnergySimulator:
    def __init__(self):
        self.generation = 50
        self.consumption = 50
        self.storage = 50
        self.max = 100

    def update(self):
        # update generation
        self.generation  += random.randint(-10, 10)
        self.storage= max(0, min(self.generation, self.max))

        # update consumption
        self.consumption += random.randint(-5, 5)
        self.consumption = max(0, min(self.consumption, self.max))

        # calc delta
        delta = self.generation - self.consumption

        # update storage
        self.storage += delta
        self.storage = max(0, min(self.storage, self.max))

    def get_data(self):
        return {
            "generation": self.generation,
            "consumption": self.consumption,
            "surplus": self.generation - self.consumption,
            "storage": self.storage
        }