import socket
import time

def send_race_data():
    host = 'localhost'
    port = 12345
    
    # Header: Event 15B
    header = "UNOFFICIAL,15B Group 1 1500m 111m,nwi,15B,1,01,15B-1-01,AUTO,7;".encode('utf-16le')
    
    # Result 1: Place 1
    result1 = "1,5,103,Heath KENNETT,BURN,2:27.93;".encode('utf-16le')
    
    # Result 2: Place 2
    result2 = "2,4,111,Connor CRAIG,CAL,2:31.53;".encode('utf-16le')

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    
    print("Sending Header (Event 15B)...")
    s.sendall(header)
    time.sleep(0.5)
    
    print("Sending Result 1...")
    s.sendall(result1)
    
    print("Sending Result 2...")
    s.sendall(result2)
    s.close()
    
    # Send separate race (Event 16)
    time.sleep(1)
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    header2 = "UNOFFICIAL,Race 16 Details,nwi,16,1,01,16-1-01,AUTO,7;".encode('utf-16le')
    result3 = "1,3,999,Another Runner,TEAM,1:00.00;".encode('utf-16le')
    
    print("Sending Header (Event 16)...")
    s.sendall(header2)
    time.sleep(0.5)
    s.sendall(result3)
    s.close()

if __name__ == "__main__":
    send_race_data()
