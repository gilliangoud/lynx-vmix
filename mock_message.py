import socket
import time

def send_lss_message():
    host = 'localhost'
    port = 12345
    
    # LSS Message Format:
    # Header: \15\00\01M\02
    header = b"\x15\x00\x01M\x02"
    
    # Message: \16\01Hello World\05
    message = b"\x16\x01Hello World\x05"
    
    # Trailer: \15\00\03\04
    trailer = b"\x15\x00\x03\04"
    
    data = header + message + trailer
    
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    s.sendall(data)
    print("Sent LSS Message")
    s.close()

if __name__ == "__main__":
    send_lss_message()
