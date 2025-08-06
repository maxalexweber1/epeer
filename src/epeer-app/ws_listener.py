from PyQt5.QtCore import QObject, pyqtSignal
import websocket
import threading
import json

from PyQt5.QtCore import QObject, pyqtSignal
import websocket
import threading
import json

class FireFlyWebSocket(QObject):
    event_received = pyqtSignal(dict)  # Signal an UI senden

    def __init__(self, subscription_name="epeer-contract", namespace="default"):
        super().__init__()
        self.subscription_name = subscription_name
        self.namespace = namespace
        self.ws = None
        self.thread = None

    def start(self):
        self.ws = websocket.WebSocketApp(
            "ws://localhost:5000/ws",           
            on_open=self.on_open,
            on_message=self.on_message,
            on_error=self.on_error,
            on_close=self.on_close
        )
        self.thread = threading.Thread(target=self.ws.run_forever)
        self.thread.daemon = True
        self.thread.start()

    def on_open(self, ws):
        print("WebSocket connected")
        start_msg = {
            "type": "start",
            "name": self.subscription_name,     
            "namespace": self.namespace,        
            "autoack": True                   
        }
        ws.send(json.dumps(start_msg))

    def on_message(self, ws, message):
        try:
            data = json.loads(message)
            if data.get("type") == "blockchain_event_received":
                print("📨 Event received:", json.dumps(data, indent=2))
                self.event_received.emit(data)  
        except Exception as e:
            print("Error decoding message:", e)

    def on_error(self, ws, error):
        print("WebSocket error: {error}")

    def on_close(self, ws, *_):
        print("WebSocket connection closed")

