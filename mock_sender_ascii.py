import socket
import time

def send_ascii_csv():
    host = 'localhost'
    port = 12345
    
    # ASCII CSV Data
    # Should NOT be parsed as time
    data = b"UNOFFICIAL,Header Info...;1,5,103,Heath KENNETT,BURN,2:27.93;2,4,111,Connor CRAIG,CAL,2:31.53;"
    
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    s.sendall(data)
    print("Sent ASCII CSV data")
    s.close()

if __name__ == "__main__":
    send_ascii_csv()
