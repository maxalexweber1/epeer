from PyQt5.QtWidgets import QTextEdit

class LogWidget(QTextEdit):
    def __init__(self):
        super().__init__()
        self.setReadOnly(True)
        self.setPlaceholderText("Action log...")

    def log(self, message: str):
        self.append(message)