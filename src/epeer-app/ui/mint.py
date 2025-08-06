from PyQt5.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QComboBox, QGroupBox
from ui.log import LogWidget
from ui.status_box import StatusBox
from ws_listener import FireFlyWebSocket

from PyQt5.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QComboBox,
    QPushButton, QGroupBox
)
from ui.log import LogWidget
from transactions import mint_tokens, burn_tokens

class MintTab(QWidget):
    def __init__(self):
        super().__init__()

        self.ws_listener = FireFlyWebSocket()
        self.ws_listener.event_received.connect(self.on_ws_event)
        self.ws_listener.start()

        main_layout = QVBoxLayout()
        self.status_box = StatusBox()

        # Mint / Burn section (middle)
        self.amount_selector = QComboBox()
        self.amount_selector.addItems(["100", "200", "300", "400", "500"])

        self.mint_button = QPushButton("Mint")
        self.burn_button = QPushButton("Burn")

        self.mint_button.clicked.connect(self.on_mint)
        self.burn_button.clicked.connect(self.on_burn)

        action_layout = QHBoxLayout()
        action_layout.addWidget(QLabel("Select amount:"))
        action_layout.addWidget(self.amount_selector)
        action_layout.addWidget(self.mint_button)
        action_layout.addWidget(self.burn_button)

        action_box = QGroupBox("Token Actions")
        action_box.setLayout(action_layout)

        self.log_widget = LogWidget()

        main_layout.addWidget(self.status_box)
        main_layout.addWidget(action_box)
        main_layout.addWidget(self.log_widget)
        self.setLayout(main_layout)

    def on_mint(self):
        quantity = int(self.amount_selector.currentText())
        tx = mint_tokens(quantity)
        self.log_widget.log(f"Mint requested: {quantity} E-Tokens Tx: {tx}")
        
    def on_burn(self):
        quantity = int(self.amount_selector.currentText())
        tx = burn_tokens(quantity)
        self.log_widget.log(f"Burn requested: {quantity} E-Tokens | Tx: {tx}")

    def on_ws_event(self, event_data):
        bc_event = event_data.get("blockchainEvent", {})
        name = bc_event.get("name")
        tx_id = bc_event.get("output", {}).get("transactionId")

        if tx_id:
            self.log_widget.log(f"Event received: {name} | TxID: {tx_id}")
        else:
            self.log_widget.log(f"Blockchain Event: {name}")




