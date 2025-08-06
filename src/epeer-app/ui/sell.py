from PyQt5.QtWidgets import QWidget, QVBoxLayout, QLabel, QLineEdit, QPushButton, QGroupBox
from ui.log import LogWidget
from ui.status_box import StatusBox

from transactions import send_sell_order

class SellTab(QWidget):
    def __init__(self):
        super().__init__()

        layout = QVBoxLayout()

        self.status_box = StatusBox()

        self.log_widget = LogWidget()

        group = QGroupBox("Sell E-Token")
        group_layout = QVBoxLayout()

        self.quantity_input = QLineEdit()
        self.quantity_input.setPlaceholderText("Quantity")
        self.price_input = QLineEdit()
        self.price_input.setPlaceholderText("Price ADA")

        self.button = QPushButton("Sell")
        self.button.clicked.connect(self.on_sell)

        group_layout.addWidget(QLabel("Quantity"))
        group_layout.addWidget(self.quantity_input)
        group_layout.addWidget(QLabel("Price:"))
        group_layout.addWidget(self.price_input)
        group_layout.addWidget(self.button)

        group.setLayout(group_layout)
        layout.addWidget(self.status_box)
        layout.addWidget(group)
        layout.addWidget(self.log_widget)
        self.setLayout(layout)

    def on_sell(self):
        quantity = int(self.quantity_input.text())
        price = int(self.price_input.text()) * 1000000
        tx = send_sell_order(price,quantity)
        self.log_widget.log(f"Sell: {quantity} E-Token listed for: {price} $ADA | Tx: {tx}")


