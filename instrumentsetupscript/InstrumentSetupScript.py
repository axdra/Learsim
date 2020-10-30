import pyautogui
import json
import time
screenWidht, screenHeight = pyautogui.size()
configLocation = "config.json"

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
#