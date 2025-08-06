from PyQt5.QtWidgets import QGroupBox, QVBoxLayout, QLabel
from status import AppStatus

class StatusBox(QGroupBox):
    def __init__(self, title="Status"):
        super().__init__(title)
        self.status = AppStatus()

        self.energy_label = QLabel()
        self.token_label = QLabel()
        self.status_label = QLabel("Loading")

        layout = QVBoxLayout()
        layout.addWidget(self.energy_label)
        layout.addWidget(self.token_label)
        layout.addWidget(self.status_label)
        
        self.setLayout(layout)

        self.status.subscribe(self.update_labels)
        self.update_labels(self.status.stored_energy, self.status.stored_token)

    def update_labels(self, energy, token):
        self.energy_label.setText(f"Stored Energy: {energy} kWh")
        self.token_label.setText(f"Current E-Tokens: {token}")
    def update_status(self, message: str):
        self.status_label.setText(message)
