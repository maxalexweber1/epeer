from network import get_e_token 

class AppStatus:
    _instance = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance.stored_energy = 1000
            try:
                token_balance = get_e_token()
            except Exception as e:
                print(f"[AppStatus] Failed to fetch token balance: {e}")
                token_balance = 0
            cls._instance.stored_token = token_balance
            cls._instance.subscribers = []
        return cls._instance

    def update(self, energy=None, token=None):
        if energy is not None:
            self.stored_energy = energy
        if token is not None:
            self.stored_token = token
        self.notify_subscribers()

    def subscribe(self, callback):
        self.subscribers.append(callback)

    def notify_subscribers(self):
        for cb in self.subscribers:
            cb(self.stored_energy, self.stored_token)
