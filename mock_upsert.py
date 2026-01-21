import socket
import time

def send_duplicate_lanes():
    host = 'localhost'
    port = 12345
    
    # Header: Event 99
    header = "UNOFFICIAL,Event 99,nwi,99,1,01,99-1-01,AUTO,7;".encode('utf-16le')
    
    # Result 1: Lane 5, Time 10.00
    result1 = "1,5,101,Runner One,TEAM,10.00;".encode('utf-16le')
    
    # Result 2: Lane 5, Time 9.99 (Correction)
    result2 = "1,5,101,Runner One,TEAM,9.99;".encode('utf-16le')

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    
    print("Sending Header (Event 99)...")
    s.sendall(header)
    time.sleep(0.5)
    
    print("Sending Result 1 (Lane 5, Time 10.00)...")
    s.sendall(result1)
    
    time.sleep(1)
    
    print("Sending Result 2 (Lane 5, Time 9.99)...")
    s.sendall(result2)
    s.close()
    
if __name__ == "__main__":
    send_duplicate_lanes()
