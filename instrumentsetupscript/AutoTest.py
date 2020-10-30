import pymem
import pymem.process
import pymem.exception
import time
import pyautogui
import json
import time
import os
import sys
from PySide2 import QtWidgets, QtGui
from os import system, name 
screenWidht, screenHeight = pyautogui.size()
configLocation = "config.json"
testADR = (0x3592528)

#0x7ff6b1ba2528

class SystemTrayIcon(QtWidgets.QSystemTrayIcon):
    def __init__(self,icon,parent=None):
        QtWidgets.QSystemTrayIcon.__init__(self,icon,parent)
        self.setToolTip("Not Monitoring Memory")

    def onTrayIconActivated(self,reason):
        if reason == self.DoubleClick:
            self.openConfig()
    def openConfig(self):
        os.system(f"code {configLocation}")
    def setNewToolTip(self, tt):
        self.setToolTip(tt)


f = open(configLocation, "r")
steps = json.loads(f.read())

def move(x,y):
    print(f'Moving to {x},{y}')
    pyautogui.moveTo(x=x,y=y)
    return True
def drag(x,y,dur):
    print(f'Draging to {x},{y} in {dur} seconds')
    pyautogui.dragTo(x,y,dur,button='left')
    return True
def popout(x,y):
    print(f'Popping out window at {x},{y}')  
    pyautogui.keyDown('i')   
    pyautogui.moveTo(x=x,y=y)
    time.sleep(0.01)   
    pyautogui.mouseDown(button='left')
    time.sleep(0.01)  
    pyautogui.mouseUp(button='left')
    time.sleep(0.01)   
    pyautogui.keyUp('i')
    return True
def mouseClick(x,y,btn):
    print(f'Pressing {btn} click at {x},{y}')
    pyautogui.moveTo(x=x,y=y)
    time.sleep(0.01)   
    pyautogui.mouseDown(button='left')
    time.sleep(0.01)  
    pyautogui.mouseUp(button='left')
    return True
def keyPress(btn,dur):
    if(dur == 0):
        print(f'Pressing {btn}')
        pyautogui.press(btn)
    else:
        print(f'Pressing {btn} for {dur} seconds')
        pyautogui.keyDown(btn)
        time.sleep(dur)
        pyautogui.keyUp(btn)
    
    return True
def delay(dur):
    print(f'Waiting for {dur}')
    time.sleep(dur)
    return True
def combKeyPress(btn1,btn2):
    print(f'Pressing for {btn1} and {btn2}')
    pyautogui.keyDown(btn1)
    time.sleep(0.1) 
    pyautogui.press(btn2)
    time.sleep(0.1) 
    pyautogui.keyUp(btn1)
    return True
def doAuto():
    for step in steps['steps']:
        if step['type'] == "move":
            move(step['x'],step['y'])
        elif step['type'] == "drag":
            drag(step['x'],step['y'],step['duration'])
        elif step['type'] == "popout":
            popout(step['x'],step['y'])     
        elif step['type'] == "mouseClick":
            mouseClick(step['x'],step['y'],step['key'])
        elif step['type'] == "keyPress":
            keyPress(step['key'],step['duration'])
        elif step['type'] == "delay":
            delay(step['duration'])
        elif step['type'] == "combinationPress":
            combKeyPress(step['key1'],step['key2'])

def ReadMemory(Proc,ProcModule):
    value = 'NaN'
    while True:
        testread = Proc.read_int(ProcModule+  0x3592528)
        
        if(testread == 256 and testread != value and value == 0):
            doAuto()
            value = testread
            print("Detected Cockpit")
        elif(testread == 0 and testread != value):
            value = 0
            print("Detected No Cockpit")
        else:
            value = testread
        time.sleep(1)    
    
        
    
    




def main():
    app = QtWidgets.QApplication(sys.argv)
    w = QtWidgets.QWidget()
    tray_icon = SystemTrayIcon(QtGui.QIcon("icon.png"),w)
    tray_icon.show()
    while True:
        try:
            
            pm = pymem.Pymem("FlightSimulator.exe")
            client = pymem.process.module_from_name(pm.process_handle, "FlightSimulator.exe").lpBaseOfDll
            system('cls') 
            print(f"Started memory monitoring...... @ {hex(client+  0x3592528)}")
            
            tray_icon.hide()
            tray_icon = SystemTrayIcon(QtGui.QIcon("iconG.png"),w)
            tray_icon.show()
            tray_icon.setNewToolTip(f"Reading memory @ {hex(client+  0x3592528)}")
            ReadMemory(pm,client)


        except pymem.exception.ProcessNotFound:
            print("Could not attach to FS2020, is it running?")
            print("Testing in 10 seconds again...")
            time.sleep(10)
            pass

if __name__ == '__main__':
    main()