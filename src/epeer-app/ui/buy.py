from PyQt5.QtWidgets import QWidget, QVBoxLayout, QComboBox, QLabel, QLineEdit, QPushButton, QGroupBox
from ui.log import LogWidget
from transactions import send_buy_order
from network import get_market_utxos
from ui.status_box import StatusBox

class BuyTab(QWidget):
    def __init__(self):
        super().__init__()

        layout = QVBoxLayout()
        self.log_widget = LogWidget()

        group = QGroupBox("Buy E-Token")
        group_layout = QVBoxLayout()

        self.status_box = StatusBox()
        self.offer_selector = QComboBox()
        self.offer_selector.addItems([])

        self.button = QPushButton("Buy")
        self.button.clicked.connect(self.on_buy)

        group_layout.addWidget(self.offer_selector)
        group_layout.addWidget(self.button)

        group.setLayout(group_layout)
        layout.addWidget(self.status_box)
        layout.addWidget(group)
        layout.addWidget(self.log_widget)
        self.setLayout(layout)

        self.utxo_map = {} 
        self.load_offers()

    def on_buy(self):
        utxo = self.utxo_input.text()
        tx = send_buy_order()
        self.log_widget.log(f"Buy Order submitted: {utxo}")

    def load_offers(self):
        self.offer_selector.clear()
        self.utxo_map.clear()

        try:
            offers = get_market_utxos()

            for i, offer in enumerate(offers):
                label = f"{offer['quantity']} E-Token for {offer['lovelace'] / 1_000_000:.2f} ADA"
                self.offer_selector.addItem(label)
                self.utxo_map[i] = offer

                self.status_box.update_status("{len(offers)} offers loaded.")
        except Exception as e:
            self.status_box.update_status(f"Error loading offers: {str(e)}")