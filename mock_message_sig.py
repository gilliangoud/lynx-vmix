import socket
import time

def send_signature_message():
    host = 'localhost'
    port = 12345
    
    # 1. Empty Message: 01 4D 02 03 04
    empty_msg = b"\x01\x4D\x02\x03\x04"
    
    # 2. Hello Message: 01 4D 02 [hello] 05 03 04
    hello_msg = b"\x01\x4D\x02hello\x05\x03\x04"
    
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    
    print("Sending Empty Message...")
    s.sendall(empty_msg)
    time.sleep(1)
    
    print("Sending Hello Message...")
    s.sendall(hello_msg)
    
    s.close()

if __name__ == "__main__":
    send_signature_message()
