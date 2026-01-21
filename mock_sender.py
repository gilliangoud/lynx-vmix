import socket
import time

def send_utf16_csv():
    host = 'localhost'
    port = 12345
    
    # Mock data based on user provided dump
    header = "UNOFFICIAL,Header Info...;"
    record1 = "1,5,103,Heath KENNETT,BURN,2:27.93 ;"
    record2 = "2,4,111,Connor CRAIG,CAL,2:31.53 ;"
    
    # Combine and encode to UTF-16LE
    data = (header + record1 + record2).encode('utf-16le')
    
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    s.sendall(data)
    print("Sent UTF-16LE data")
    s.close()

if __name__ == "__main__":
    send_utf16_csv()
