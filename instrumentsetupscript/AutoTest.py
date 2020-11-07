import pymem
import pymem.process
import pymem.exception
import time
import pyautogui
import json
import time
import os
import sys
import threading
from PySide2 import QtWidgets, QtGui
from os import system, name 

screenWidht, screenHeight = pyautogui.size()
configLocation = "config.json"
testADR = (0x3592528)

#0x7ff6b1ba2528

class SystemTrayIcon(QtWidgets.QSystemTrayIcon):
    def __init__(self,icon,parent=None):
        QtWidgets.QSystemTrayIcon.__init__(self,icon,parent)
        menu = QtWidgets.QMenu(parent)

        toggleMemoryRead = menu.addAction("Toggle Memory Reading")
        toggleMemoryRead.triggered.connect(self.toggleReading)
        
        openConfig = menu.addAction("Open Configuration")
        openConfig.triggered.connect(self.open_config)



        exitApp = menu.addAction("Exit")
        exitApp.triggered.connect(self.exit)


        menu.addSeparator()
        self.setContextMenu(menu)
        self.activated.connect(self.onTrayIconActivated)

    def onTrayIconActivated(self, reason):
        """
        This function will trigger function on click or double click
        :param reason:
        :return:
        """
        if reason == self.DoubleClick:
            self.open_notepad()
        # if reason == self.Trigger:
        #     self.open_notepad()
    def exit(self, reason):
        self.thread.exitThread()
        sys.exit()
    def setThread(self, thread):
        self.thread = thread
    def open_config(self):
        os.system('open config.json')

    def toggleReading(self):
        self.thread.togglePauseReading()
    def setNewToolTip(self, tt):
        self.setToolTip(tt)
    def setNewIcon(self,icon):
        self.setIcon(icon)


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


   
    
class MemoryRead (threading.Thread):
    def __init__(self,trayicon,w):
        threading.Thread.__init__(self)
        self.tray_icon = trayicon
        self.w = w
        self.RunThread = True
        self.paused = False
    def run(self):
        while self.RunThread:
            if not self.paused:
                try:
                    
                    pm = pymem.Pymem("FlightSimulator.exe")
                    client = pymem.process.module_from_name(pm.process_handle, "FlightSimulator.exe").lpBaseOfDll
                    system('cls') 
                    print(f"Started memory monitoring...... @ {hex(client+  0x3592528)}")

                    self.tray_icon.hide()
                    self.tray_icon.setNewIcon(QtGui.QIcon("iconG.png"))
                    self.tray_icon.show()
                    self.tray_icon.setNewToolTip(f"Reading memory @ {hex(client+  0x3592528)}")
                    self.ReadMemory(pm,client)
                    self.tray_icon.hide()
                    self.tray_icon.setNewIcon(QtGui.QIcon("icon.png"))
                    self.tray_icon.show()
                    self.tray_icon.setNewToolTip(f"Not Monitoring Memory")

                except pymem.exception.ProcessNotFound:
                    print("Could not attach to FS2020, is it running?")
                    print("Testing in 10 seconds again...")
                    time.sleep(10)
                    pass
    
    def exitThread(self):
        self.RunThread = False
    
    def togglePauseReading (self):
        
        self.paused = not (bool(self.paused))
        if(self.paused):
            self.tray_icon.hide()
            self.tray_icon.setIcon(QtGui.QIcon("icon.png"))
            self.tray_icon.show()
            self.tray_icon.setNewToolTip(f"Not Monitoring Memory (Paused)")
        else:
            self.tray_icon.setNewToolTip(f"Not Monitoring Memory")
    
    def ReadMemory(self,Proc,ProcModule):
        value = 'NaN'
        try:
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
        except:
            return
        
    




def main():
    app = QtWidgets.QApplication(sys.argv)
    w = QtWidgets.QWidget()
    tray_icon = SystemTrayIcon(QtGui.QIcon("icon.png"),w)
    mR = MemoryRead(tray_icon,w)
    mR.start()
    tray_icon.setThread(mR)
    tray_icon.show()
    sys.exit(app.exec_())

    


if __name__ == '__main__':
    main()

