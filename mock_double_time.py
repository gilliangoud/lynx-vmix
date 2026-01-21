import socket
import time

def send_double_time():
    host = 'localhost'
    port = 12345
    
    # Simulate concatenated time packets
    # "  12:26:01.2  " + "  12:26:01.3  "
    packet1 = b"  12:26:01.2  "
    packet2 = b"  12:26:01.3  "
    data = packet1 + packet2
    
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    s.sendall(data)
    print("Sent double time packet")
    s.close()
    
    time.sleep(1)
    
    # Send normal packet to check recovery
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    s.sendall(b"  12:26:01.4  ")
    s.close()

if __name__ == "__main__":
    send_double_time()
