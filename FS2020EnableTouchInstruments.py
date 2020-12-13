import win32api
import win32gui
import time
import keyboard
import pyautogui
import sys
leftKey = win32api.GetKeyState(0x01)  
rightKey = win32api.GetKeyState(0x02) 
readTouch = False
running = True
safeSpace = False
#Set p1 as top left cord and p2 as bottom right cord [x,y]
p1 = [100,100]
p2 = [400,400]
def toggle():
    global readTouch
    readTouch = not readTouch
def exitapp():
    global running
    running = False
def toggleSafeSpace():
    global safeSpace
    safeSpace = not safeSpace

keyboard.add_hotkey('ctrl+f4', toggle)
keyboard.add_hotkey('ctrl+f5', exitapp)
keyboard.add_hotkey('ctrl+f6', toggleSafeSpace)


while True:
    if not running:
        break
    if(readTouch):
        leftRead = win32api.GetKeyState(0x01)
        rightRead = win32api.GetKeyState(0x02)
        

        if leftRead != leftKey:
            leftKey = leftRead
         
            if leftRead < 0:
                point = win32gui.GetCursorPos()
                
                #print(point,"l")
                if(not safeSpace or (point[0] >= p1[0] and point[0] <= p2[0] and point[1] >= p1[1] and point[1] <= p2[1])):
                    print("yee")
                    pyautogui.click(button='left', x=point[0]+1, y=point[1]+1) 
        if rightRead != rightKey: 
            rightKey = rightRead
            if rightRead < 0:
                point = win32gui.GetCursorPos()
                #print(point,"r")
                pyautogui.mouseDown(button='right', x=point[0]+1, y=point[1]+1) 


    time.sleep(0.001)
