import win32api
import win32gui
import time
import keyboard
import pyautogui
import sys
leftKey = win32api.GetKeyState(0x01)  # Left button down = 0 or 1. Button up = -127 or -128
rightKey = win32api.GetKeyState(0x02)  # Right button down = 0 or 1. Button up = -127 or -128
readTouch = False
running = True

def toggle():
    global readTouch
    readTouch = not readTouch
def exitapp():
    global running
    running = False

keyboard.add_hotkey('ctrl+f4', toggle)
keyboard.add_hotkey('ctrl+f5', exitapp)


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
                pyautogui.click(button='left', x=point[0], y=point[1]) 
        if rightRead != rightKey: 
            rightKey = rightRead
            if rightRead < 0:
                point = win32gui.GetCursorPos()
                #print(point,"r")
                pyautogui.mouseDown(button='right', x=point[0], y=point[1]) 


    time.sleep(0.001)