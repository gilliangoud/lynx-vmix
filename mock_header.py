import socket
import time

def send_header():
    host = 'localhost'
    port = 12345
    
    # Header from user log:
    # "UNOFFICIAL,15B Group 1 1500m 111m,nwi,15B,1,01,15B-1-01,AUTO,7"
    # We send this as UTF-16LE CSV since that seems to be the format containing it.
    header = "UNOFFICIAL,15B Group 1 1500m 111m,nwi,15B,1,01,15B-1-01,AUTO,7;"
    
    data = header.encode('utf-16le')
    
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    s.sendall(data)
    print("Sent Header")
    s.close()

if __name__ == "__main__":
    send_header()
