from PyQt5.QtWidgets import QMainWindow, QTabWidget
from ui.sell import SellTab
from ui.mint import MintTab
from ui.buy import BuyTab

class EPeerApp(QMainWindow):
    def __init__(self):
        super().__init__()

        self.setWindowTitle("FireFly E-Peer UI")
        self.resize(800, 500)
        self.tabs = QTabWidget()
        self.tabs.addTab(MintTab(), "Mint / Redeem")
        self.tabs.addTab(SellTab(), "Sell")
        self.tabs.addTab(BuyTab(), "Buy")
        self.setCentralWidget(self.tabs)

