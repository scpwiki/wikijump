import java.security.*;
import java.applet.Applet;
import java.awt.*;
import java.util.*;
import java.awt.event.*;
import netscape.javascript.*;
import java.io.*;
import java.lang.reflect.*;
import java.net.URL;

public final class DOHRobot extends Applet{
	// order of execution:
	// wait for user to trust applet
	// load security manager to prevent Safari hang
	// discover document root in screen coordinates
	// discover keyboard capabilities
	// tell doh to continue with the test

	// link to doh
	// To invoke doh, call eval with window.eval("jsexp")
	// Note that the "window" is an iframe!
	// You might need to break out of the iframe with an intermediate function
	// in the parent window.
	private JSObject window = null;

	// java.awt.Robot
	// drives the test
	// you need to sign the applet JAR for this to work
	private Robot robot = null;

	// In order to preserve the execution order of Robot commands,
	// we have to serialize commands by having them join() the previous one.
	// Otherwise, if you run doh.robot.typeKeys("dijit"), you frequently get something
	// like "diijt"
	private static Thread previousThread = null;

	// Keyboard discovery.
	// At init, the Robot types keys into a textbox and JavaScript tells the
	// Robot what it got back.
	// charMap maps characters to the KeyEvent that generates the character on
	// the user's machine.
	// charMap uses the Java 1.4.2 (lack of) template syntax for wider
	// compatibility.
	private static HashMap charMap = null;
	// Java key constants to iterate over
	// not all are available on all machines!
	private Vector vkKeys = null;
	// some state variables
	private boolean shift = false;
	private boolean altgraph = false;
	private boolean ctrl = false;
	private boolean alt = false;
	// shake hands with JavaScript the first keypess to wake up FF2/Mac
	private boolean jsready = false;
	private String keystring = "";

	// Firebug gets a little too curious about our applet for its own good
	// setting firebugIgnore to true ensures Firebug doesn't break the applet
	public boolean firebugIgnore = true;

	private SecurityManager securitymanager;
	private double key = -1;

	// The screen x,y of the document upper left corner.
	// We only set it once so people are less likely to take it over.
	private boolean inited = false;
	private int docScreenX = -100;
	private int docScreenY = -100;
	private int docScreenXMax;
	private int docScreenYMax;

	// The last reported mouse x,y.
	// If this is different from the real one, something's up.
	private int lastMouseX;
	private int lastMouseY;

	// save a pointer to doh.robot for fast access
	JSObject dohrobot = null;

	// java.awt.Applet methods
	public void stop(){
		window = null;
		// only secure code run once
		if(key != -2){
			// prevent further execution of secure functions
			key = -2;
			// Java calls this when you close the window.
			// It plays nice and restores the old security manager.
			AccessController.doPrivileged(new PrivilegedAction(){
				public Object run(){
					log("Stop");
					securitymanager.checkTopLevelWindow(null);
					log("Security manager reset");
					return null;
				}
			});
		}
	}

	final private class onvisible extends ComponentAdapter{
		public void componentShown(ComponentEvent evt){
			// sets the security manager to fix a bug in liveconnect in Safari on Mac
			if(key != -1){ return; }
			window = (JSObject) JSObject.getWindow(applet());   

			AccessController.doPrivileged(new PrivilegedAction(){
				public Object run(){
					log("> init Robot");
					try{
						SecurityManager oldsecurity = System.getSecurityManager();
						boolean isOpera = false;
						try{
							isOpera = (System.getProperty("browser").equals("Opera.plugin"));
						}catch(Exception e){}
						try{
							securitymanager = oldsecurity;
							securitymanager.checkTopLevelWindow(null);
							// xdomain
							if(charMap == null){
								if(!confirm("DOH has detected that the current Web page is attempting to access DOH, but belongs to a different domain than the one you agreed to let DOH automate. If you did not intend to start a new DOH test by visiting this Web page, press Cancel now and leave the Web page. Otherwise, press OK to trust this domain to automate DOH tests.")){
									stop();
									return null;
								}
							}
							log("Found old security manager");
						}catch(Exception e){
							e.printStackTrace();
							log("Making new security manager");
							securitymanager = new RobotSecurityManager(isOpera,
									oldsecurity);
							securitymanager.checkTopLevelWindow(null);
							System.setSecurityManager(securitymanager);
						}
						// instantiate the Robot
						robot = new Robot();
					}catch(Exception e){
						log("Error calling _init_: "+e.getMessage());
						key = -2;
						e.printStackTrace();
					}
					log("< init Robot");
					return null;
				}
			});
			if(key == -2){
				// applet not trusted
				// start the test without it
				window.eval("doh.robot._appletDead=true;doh.run();");
			}else{
				// now that the applet has really started, let doh know it's ok to use it
				log("_initRobot");
				dohrobot = (JSObject) window.eval("doh.robot");
				dohrobot.call("_initRobot", new Object[]{ applet() });
			}
		}
	}

	public void init(){
		// ensure isShowing = true
		addComponentListener(new onvisible());
	}

	// loading functions
	public void _setKey(double key){
		if(key == -1){
			return;
		}else if(this.key == -1){
			this.key = key;
		}
	}

	private boolean isSecure(double key){
		boolean result = this.key != -1 && this.key == key;
		log("Key secure: " + result);
		return result;
	}

	public void _callLoaded(double sec){
		log("> _callLoaded Robot");
		if(!isSecure(sec)){
			return;
		}
		Thread thread = new Thread(){
			public void run(){
				AccessController.doPrivileged(new PrivilegedAction(){
					public Object run(){
						Point p = getLocationOnScreen();
						log("Document root: ~"+p.toString());
						int x = p.x + 16;
						int y = p.y + 8;
						// click the mouse over the text box
						try{
							Thread.sleep(100);
						}catch(Exception e){};
						robot.mouseMove(x, y);
						try{
							Thread.sleep(100);
						}catch(Exception e){};
						robot.mousePress(InputEvent.BUTTON1_MASK);
						try{
							Thread.sleep(100);
						}catch(Exception e){}
						robot.mouseRelease(InputEvent.BUTTON1_MASK);
						try{
							Thread.sleep(100);
						}catch(Exception e){}
						log("< _callLoaded Robot");
						return null;
					}
				});
			}
		};
		thread.start();
	}

	// convenience functions
	private DOHRobot applet(){
		return this;
	}

	public void log(final String s){
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				System.out.println((new Date()).toString() + ": " + s);
				return null;
			}
		});
	}

	private void alert(final String s){
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				window.eval("top.alert(\"" + s + "\");");
				return null;
			}
		});
	}

	private boolean confirm(final String s){
		return ((Boolean) AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				return ((Boolean) window.eval("top.confirm(\"" + s + "\");"));
			}
		})).booleanValue();
	}

	// mouse discovery code
	public void setDocumentBounds(double sec, int x, int y, int w, int h){
		log("> setDocumentBounds");
		if(!isSecure(sec))
			return;
		// call from JavaScript
		// tells the Robot where the screen x,y of the upper left corner of the
		// document are
		// not screenX/Y of the window; really screenLeft/Top in IE, but not all
		// browsers have this
		if(!inited){
			inited = true;
			this.docScreenX = x;
			this.docScreenY = y;
			this.docScreenXMax = x + w;
			this.docScreenYMax = y + h;
		}
		log("< setDocumentBounds");
	}

	// keyboard discovery code
	private void _mapKey(char charCode, int keyindex, boolean shift,
			boolean altgraph){
		log("_mapKey: " + charCode);
		// if character is not in map, add it
		if(!charMap.containsKey(new Integer(charCode))){
			log("Notified: " + (char) charCode);
			KeyEvent event = new KeyEvent(applet(), 0, 0,
					(shift ? KeyEvent.SHIFT_MASK : 0)
							+ (altgraph ? KeyEvent.ALT_GRAPH_MASK : 0),
					((Integer) vkKeys.get(keyindex)).intValue(),
					(char) charCode);
			charMap.put(new Integer(charCode), event);
			log("Mapped char " + (char) charCode + " to KeyEvent " + event);
			if(((char) charCode) >= 'a' && ((char) charCode) <= 'z'){
				// put shifted version of a-z in automatically
				int uppercharCode = (int) Character
						.toUpperCase((char) charCode);
				event = new KeyEvent(applet(), 0, 0, KeyEvent.SHIFT_MASK
						+ (altgraph ? KeyEvent.ALT_GRAPH_MASK : 0),
						((Integer) vkKeys.get(keyindex)).intValue(),
						(char) uppercharCode);
				charMap.put(new Integer(uppercharCode), event);
				log("Mapped char " + (char) uppercharCode + " to KeyEvent "
						+ event);
			}
		}
	}

	public void _notified(double sec, final String chars){
		if(!isSecure(sec))
			return;
		// decouple from JavaScript; thread join could hang it
		Thread thread = new Thread("_notified"){
			public void run(){
				AccessController.doPrivileged(new PrivilegedAction(){
					public Object run(){
						try{
							// wait for release shift/altgraph to resolve
							if(previousThread != null){
								previousThread.join();
							}
						}catch(Exception e){
						}
						keystring += chars;
						if(altgraph && !shift){
							shift = false;
							// Set robot auto delay now that FF/Mac inited all of the keys. 
							// Good for DND.
							robot.setAutoDelay(1);
							try{
								log(keystring);
								int index = 0;
								for (int i = 0; (i < vkKeys.size())
										&& (index < keystring.length()); i++){
									char c = keystring.charAt(index++);
									_mapKey(c, i, false, false);
								}
								for (int i = 0; (i < vkKeys.size())
										&& (index < keystring.length()); i++){
									char c = keystring.charAt(index++);
									_mapKey(c, i, true, false);
								}
								for (int i = 0; (i < vkKeys.size())
										&& (index < keystring.length()); i++){
									char c = keystring.charAt(index++);
									_mapKey(c, i, false, true);
								}
								// notify DOH that the applet finished init
								dohrobot.call("_onKeyboard", new Object[]{});
							}catch(Exception e){
								e.printStackTrace();
							}
							return null;
						}else if(!shift){
							shift = true;
						}else{
							shift = false;
							altgraph = true;
						}
						pressNext();
						// }
						return null;
					}
				});
			}
		};
		thread.start();
	}

	private void pressNext(){
		final Thread myPreviousThread = previousThread;
		Thread thread = new Thread("pressNext"){
			public void run(){
				try{
					// wait for release shift/altgraph to resolve
					if(myPreviousThread != null){
						myPreviousThread.join();
					}
				}catch(Exception e){
				}
				// first time, press shift (have to do it here instead of
				// _notified to avoid IllegalThreadStateException on Mac)
				log("starting up, " + shift + " " + altgraph);
				if(shift){
					robot.keyPress(KeyEvent.VK_SHIFT);
					log("Pressing shift");
				}
				try{
					if(altgraph){
						robot.keyPress(KeyEvent.VK_ALT_GRAPH);
						log("Pressing alt graph");
					}
				}catch(Exception e){
					log("Error pressing alt graph");
					e.printStackTrace();
					_notified(key, "");
					return;
				}
				dohrobot.call("_nextKeyGroup", new Object[]{ new Integer(vkKeys.size()) });
				for (int keyindex = 0; keyindex < vkKeys.size(); keyindex++){
					try{
						log("Press "
								+ ((Integer) vkKeys.get(keyindex)).intValue());
						robot.keyPress(((Integer) vkKeys.get(keyindex))
								.intValue());
						log("Release "
								+ ((Integer) vkKeys.get(keyindex)).intValue());
						robot.keyRelease(((Integer) vkKeys.get(keyindex))
								.intValue());
						if(altgraph && (keyindex == (vkKeys.size() - 1))){
							robot.keyRelease(KeyEvent.VK_ALT_GRAPH);
							log("Releasing alt graph");
						}
						if(shift && (keyindex == (vkKeys.size() - 1))){
							robot.keyRelease(KeyEvent.VK_SHIFT);
							log("Releasing shift");
						}
					}catch(Exception e){
					}
					try{
						log("Press space");
						robot.keyPress(KeyEvent.VK_SPACE);
						log("Release space");
						robot.keyRelease(KeyEvent.VK_SPACE);
					}catch(Exception e){
						e.printStackTrace();
					}
				}
			}
		};
		previousThread = thread;
		thread.start();
	}

	public void _initWheel(double sec){
		log("> initWheel");
		if(!isSecure(sec))
			return;
		Thread thread=new Thread(){
			public void run(){
				// calibrate the mouse wheel now that textbox is focused
				int dir=1;
				if(System.getProperty("os.name").toUpperCase().indexOf("MAC") != -1){
					dir=-1;
				}
				robot.mouseWheel(dir);
				try{
					Thread.sleep(100);
				}catch(Exception e){}
				log("< initWheel");
			}
		};
		thread.start();
	}

	public void _initKeyboard(double sec){
		log("> initKeyboard");
		// javascript entry point to discover the keyboard
		if(!isSecure(sec))
			return;
		if(charMap != null){
			dohrobot.call("_onKeyboard", new Object[]{});
			return;
		}

		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				charMap = new HashMap();
				KeyEvent event = new KeyEvent(applet(), 0, 0, 0,
						KeyEvent.VK_SPACE, ' ');
				charMap.put(new Integer(32), event);
				try{
					// a-zA-Z0-9 + 29 others
					vkKeys = new Vector();
					for (char i = 'a'; i <= 'z'; i++){
						vkKeys.add(new Integer(KeyEvent.class.getField(
								"VK_" + Character.toUpperCase((char) i))
								.getInt(null)));
					}
					for (char i = '0'; i <= '9'; i++){
						vkKeys.add(new Integer(KeyEvent.class.getField(
								"VK_" + Character.toUpperCase((char) i))
								.getInt(null)));
					}
					int[] mykeys = new int[]{ KeyEvent.VK_COMMA,
							KeyEvent.VK_MINUS, KeyEvent.VK_PERIOD,
							KeyEvent.VK_SLASH, KeyEvent.VK_SEMICOLON,
							KeyEvent.VK_LEFT_PARENTHESIS,
							KeyEvent.VK_NUMBER_SIGN, KeyEvent.VK_PLUS,
							KeyEvent.VK_RIGHT_PARENTHESIS,
							KeyEvent.VK_UNDERSCORE,
							KeyEvent.VK_EXCLAMATION_MARK, KeyEvent.VK_DOLLAR,
							KeyEvent.VK_CIRCUMFLEX, KeyEvent.VK_AMPERSAND,
							KeyEvent.VK_ASTERISK, KeyEvent.VK_QUOTEDBL,
							KeyEvent.VK_LESS, KeyEvent.VK_GREATER,
							KeyEvent.VK_BRACELEFT, KeyEvent.VK_BRACERIGHT,
							KeyEvent.VK_COLON, KeyEvent.VK_BACK_QUOTE,
							KeyEvent.VK_QUOTE, KeyEvent.VK_OPEN_BRACKET,
							KeyEvent.VK_BACK_SLASH, KeyEvent.VK_CLOSE_BRACKET,
							KeyEvent.VK_EQUALS };
					for (int i = 0; i < mykeys.length; i++){
						vkKeys.add(new Integer(mykeys[i]));
					}
				}catch(Exception e){
					e.printStackTrace();
				}
				Thread thread = new Thread(){
					public void run(){
						robot.setAutoDelay(0);
						log("< initKeyboard");
						pressNext();
					}
				};
				thread.start();
				return null;
			}
		});
	}

	public void typeKey(double sec, final int charCode, final int keyCode,
			final boolean alt, final boolean ctrl, final boolean shift,
			final int delay, final boolean async){
		if(!isSecure(sec))
			return;
		// called by doh.robot._keyPress
		// see it for details
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				try{
					log("> typeKey Robot " + charCode + ", " + keyCode + ", " + async);
					KeyPressThread thread = new KeyPressThread(charCode,
							keyCode, alt, ctrl, shift, delay, async?null:previousThread);
					previousThread = async?previousThread:thread;
					thread.start();
					log("< typeKey Robot");
				}catch(Exception e){
					log("Error calling typeKey");
					e.printStackTrace();
				}
				return null;
			}
		});
	}

	public void upKey(double sec, final int charCode, final int keyCode, final int delay){
		// called by doh.robot.keyDown
		// see it for details
		// a nice name like "keyUp" is reserved in Java
		if(!isSecure(sec))
			return;
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				log("> upKey Robot " + charCode + ", " + keyCode);
				KeyUpThread thread = new KeyUpThread(charCode, keyCode, delay, previousThread);
				previousThread = thread;
				thread.start();
				log("< upKey Robot");
				return null;
			}
		});
	}

	public void downKey(double sec, final int charCode, final int keyCode, final int delay){
		// called by doh.robot.keyUp
		// see it for details
		// a nice name like "keyDown" is reserved in Java
		if(!isSecure(sec))
			return;
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				log("> downKey Robot " + charCode + ", " + keyCode);
				KeyDownThread thread = new KeyDownThread(charCode, keyCode, delay, previousThread);
				previousThread = thread;
				thread.start();
				log("< downKey Robot");
				return null;
			}
		});
	}

	public void pressMouse(double sec, final boolean left,
			final boolean middle, final boolean right, final int delay){
		if(!isSecure(sec))
			return;
		// called by doh.robot.mousePress
		// see it for details
		// a nice name like "mousePress" is reserved in Java
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				log("> mousePress Robot " + left + ", " + middle + ", " + right);
				MousePressThread thread = new MousePressThread(
						(left ? InputEvent.BUTTON1_MASK : 0)
								+ (middle ? InputEvent.BUTTON2_MASK : 0)
								+ (right ? InputEvent.BUTTON3_MASK : 0), delay,
						previousThread);
				previousThread = thread;
				thread.start();
				log("< mousePress Robot");
				return null;
			}
		});
	}

	public void releaseMouse(double sec, final boolean left,
			final boolean middle, final boolean right, final int delay){
		if(!isSecure(sec))
			return;
		// called by doh.robot.mouseRelease
		// see it for details
		// a nice name like "mouseRelease" is reserved in Java
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				log("> mouseRelease Robot " + left + ", " + middle + ", "
						+ right);
				MouseReleaseThread thread = new MouseReleaseThread(
						(left ? InputEvent.BUTTON1_MASK : 0)
								+ (middle ? InputEvent.BUTTON2_MASK : 0)
								+ (right ? InputEvent.BUTTON3_MASK : 0), delay,
						previousThread);
				previousThread = thread;
				thread.start();
				log("< mouseRelease Robot");
				return null;
			}
		});
	}

	public void moveMouse(double sec, final int x1, final int y1, final int d, final int duration){
		// called by doh.robot.mouseMove
		// see it for details
		// a nice name like "mouseMove" is reserved in Java
		if(!isSecure(sec))
			return;
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				int x = x1 + docScreenX;
				int y = y1 + docScreenY;
				if(x > docScreenXMax || y > docScreenYMax){
					// TODO: try to scroll view
					log("Request to mouseMove denied");
					return null;
				}
				int delay = d;
				log("> mouseMove Robot " + x + ", " + y);
				MouseMoveThread thread = new MouseMoveThread(x, y, delay,
						duration, previousThread);
				previousThread = thread;
				thread.start();
				log("< mouseMove Robot");
				return null;
			}
		});
	}

	public void wheelMouse(double sec, final int amount, final int delay, final int duration){
		// called by doh.robot.mouseWheel
		// see it for details
		if(!isSecure(sec))
			return;
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				MouseWheelThread thread = new MouseWheelThread(amount, delay, duration,
						previousThread);
				previousThread = thread;
				thread.start();
				return null;
			}
		});
	}

	private int getVKCode(int charCode, int keyCode){
		int keyboardCode = 0;
		if(charCode >= 32){
			// if it is printable, then it lives in our hashmap
			KeyEvent event = (KeyEvent) charMap.get(new Integer(charCode));
			keyboardCode = event.getKeyCode();
		}
		else{
			switch (keyCode){
				case 13:
					keyboardCode = KeyEvent.VK_ENTER;
					break;
				case 8:
					keyboardCode = KeyEvent.VK_BACK_SPACE;
					break;
				case 25:// shift tab for Safari
				case 9:
					keyboardCode = KeyEvent.VK_TAB;
					break;
				case 12:
					keyboardCode = KeyEvent.VK_CLEAR;
					break;
				case 16:
					keyboardCode = KeyEvent.VK_SHIFT;
					break;
				case 17:
					keyboardCode = KeyEvent.VK_CONTROL;
					break;
				case 18:
					keyboardCode = KeyEvent.VK_ALT;
					break;
				case 63250:
				case 19:
					keyboardCode = KeyEvent.VK_PAUSE;
					break;
				case 20:
					keyboardCode = KeyEvent.VK_CAPS_LOCK;
					break;
				case 27:
					keyboardCode = KeyEvent.VK_ESCAPE;
					break;
				case 32:
					log("it's a space");
					keyboardCode = KeyEvent.VK_SPACE;
					break;
				case 63276:
				case 33:
					keyboardCode = KeyEvent.VK_PAGE_UP;
					break;
				case 63277:
				case 34:
					keyboardCode = KeyEvent.VK_PAGE_DOWN;
					break;
				case 63275:
				case 35:
					keyboardCode = KeyEvent.VK_END;
					break;
				case 63273:
				case 36:
					keyboardCode = KeyEvent.VK_HOME;
					break;

				/**
				 * Constant for the <b>left</b> arrow key.
				 */
				case 63234:
				case 37:
					keyboardCode = KeyEvent.VK_LEFT;
					break;

				/**
				 * Constant for the <b>up</b> arrow key.
				 */
				case 63232:
				case 38:
					keyboardCode = KeyEvent.VK_UP;
					break;

				/**
				 * Constant for the <b>right</b> arrow key.
				 */
				case 63235:
				case 39:
					keyboardCode = KeyEvent.VK_RIGHT;
					break;

				/**
				 * Constant for the <b>down</b> arrow key.
				 */
				case 63233:
				case 40:
					keyboardCode = KeyEvent.VK_DOWN;
					break;
				case 63272:
				case 46:
					keyboardCode = KeyEvent.VK_DELETE;
					break;
				case 63289:
				case 144:
					keyboardCode = KeyEvent.VK_NUM_LOCK;
					break;
				case 63249:
				case 145:
					keyboardCode = KeyEvent.VK_SCROLL_LOCK;
					break;

				/** Constant for the F1 function key. */
				case 63236:
				case 112:
					keyboardCode = KeyEvent.VK_F1;
					break;

				/** Constant for the F2 function key. */
				case 63237:
				case 113:
					keyboardCode = KeyEvent.VK_F2;
					break;

				/** Constant for the F3 function key. */
				case 63238:
				case 114:
					keyboardCode = KeyEvent.VK_F3;
					break;

				/** Constant for the F4 function key. */
				case 63239:
				case 115:
					keyboardCode = KeyEvent.VK_F4;
					break;

				/** Constant for the F5 function key. */
				case 63240:
				case 116:
					keyboardCode = KeyEvent.VK_F5;
					break;

				/** Constant for the F6 function key. */
				case 63241:
				case 117:
					keyboardCode = KeyEvent.VK_F6;
					break;

				/** Constant for the F7 function key. */
				case 63242:
				case 118:
					keyboardCode = KeyEvent.VK_F7;
					break;

				/** Constant for the F8 function key. */
				case 63243:
				case 119:
					keyboardCode = KeyEvent.VK_F8;
					break;

				/** Constant for the F9 function key. */
				case 63244:
				case 120:
					keyboardCode = KeyEvent.VK_F9;
					break;

				/** Constant for the F10 function key. */
				case 63245:
				case 121:
					keyboardCode = KeyEvent.VK_F10;
					break;

				/** Constant for the F11 function key. */
				case 63246:
				case 122:
					keyboardCode = KeyEvent.VK_F11;
					break;

				/** Constant for the F12 function key. */
				case 63247:
				case 123:
					keyboardCode = KeyEvent.VK_F12;
					break;

				/**
				 * Constant for the F13 function key.
				 * 
				 * @since 1.2
				 */
				/*
				 * F13 - F24 are used on IBM 3270 keyboard; break; use
				 * random range for constants.
				 */
				case 124:
					keyboardCode = KeyEvent.VK_F13;
					break;

				/**
				 * Constant for the F14 function key.
				 * 
				 * @since 1.2
				 */
				case 125:
					keyboardCode = KeyEvent.VK_F14;
					break;

				/**
				 * Constant for the F15 function key.
				 * 
				 * @since 1.2
				 */
				case 126:
					keyboardCode = KeyEvent.VK_F15;
					break;

				case 63302:
				case 45:
					keyboardCode = KeyEvent.VK_INSERT;
					break;
				case 47:
					keyboardCode = KeyEvent.VK_HELP;
					break;

			}
		}
		log("Attempting to type " + (char) charCode + ":"
				+ charCode + " " + keyCode);
		log("Converted to " + keyboardCode);
		return keyboardCode;
	}

	private boolean isUnsafe(int keyboardCode){
		// run through exemption list
		log("ctrl: "+ctrl+", alt: "+alt+", shift: "+shift);
		if(((ctrl || alt) && keyboardCode == KeyEvent.VK_ESCAPE)
							|| (alt && keyboardCode == KeyEvent.VK_TAB)
							|| (ctrl && alt && keyboardCode == KeyEvent.VK_DELETE)){
			log("You are not allowed to press this key combination!");
			return true;
		}else{
			log("Safe to press.");
			return false;
		}
	}

	private void _typeKey(final int cCode, final int kCode, final boolean a,
			final boolean c, final boolean s){
		AccessController.doPrivileged(new PrivilegedAction(){
			public Object run(){
				int charCode = cCode;
				int keyCode = kCode;
				boolean alt = a;
				boolean ctrl = c;
				boolean shift = s;
				boolean altgraph = false;
				log("> _typeKey Robot " + charCode + ", " + keyCode);
				try{
					int keyboardCode=getVKCode(charCode, keyCode);
					if(charCode >= 32){
						// if it is printable, then it lives in our hashmap
						KeyEvent event = (KeyEvent) charMap.get(new Integer(charCode));
						// see if we need to press shift to generate this
						// character
						if(!shift){
							shift = event.isShiftDown();
						}
						altgraph = event.isAltGraphDown();
						keyboardCode = event.getKeyCode();
					}

					// run through exemption list
					if(!isUnsafe(keyboardCode)){
						if(shift){
							log("Pressing shift");
							robot.keyPress(KeyEvent.VK_SHIFT);
						}
						if(alt){
							log("Pressing alt");
							robot.keyPress(KeyEvent.VK_ALT);
						}
						if(altgraph){
							log("Pressing altgraph");
							robot.keyPress(KeyEvent.VK_ALT_GRAPH);
						}
						if(ctrl){
							log("Pressing ctrl");
							robot.keyPress(KeyEvent.VK_CONTROL);
						}
						if(keyboardCode != KeyEvent.VK_SHIFT
								&& keyboardCode != KeyEvent.VK_ALT
								&& keyboardCode != KeyEvent.VK_CONTROL){
							try{
								robot.keyPress(keyboardCode);
								robot.keyRelease(keyboardCode);
							}catch(Exception e){
								log("Error while actually typing a key");
								e.printStackTrace();
							}

						}
						if(ctrl){
							robot.keyRelease(KeyEvent.VK_CONTROL);
							ctrl = false;
						}
						if(alt){
							robot.keyRelease(KeyEvent.VK_ALT);
							alt = false;
						}
						if(altgraph){
							robot.keyRelease(KeyEvent.VK_ALT_GRAPH);
							altgraph = false;
						}
						if(shift){
							log("Releasing shift");
							robot.keyRelease(KeyEvent.VK_SHIFT);
							shift = false;
						}
					}
				}catch(Exception e){
					log("Error in _typeKey");
					e.printStackTrace();
				}
				log("< _typeKey Robot");
				return null;
			}
		});
	}

	public boolean hasFocus(){
		try{
			return ((Boolean) window
					.eval("var result=false;if(window.parent.document.hasFocus){result=window.parent.document.hasFocus();}else{result=true;}result;"))
					.booleanValue();
		}catch(Exception e){
			// runs even after you close the window!
			return false;
		}
	}

	// Threads for common Robot tasks
	// (so as not to tie up the browser rendering thread!)
	// declared inside so they have private access to the robot
	// we do *not* want to expose that guy!
	final private class KeyPressThread extends Thread{
		private int charCode;
		private int keyCode;
		private boolean alt;
		private boolean ctrl;
		private boolean shift;
		private int delay;
		private Thread myPreviousThread = null;

		public KeyPressThread(int charCode, int keyCode, boolean alt,
				boolean ctrl, boolean shift, int delay, Thread myPreviousThread){
			log("KeyPressThread constructor " + charCode + ", " + keyCode);
			this.charCode = charCode;
			this.keyCode = keyCode;
			this.alt = alt;
			this.ctrl = ctrl;
			this.shift = shift;
			this.delay = delay;
			this.myPreviousThread = myPreviousThread;
		}

		public void run(){
			try{
				if(myPreviousThread != null)
					myPreviousThread.join();
				// in different order so async works
				while(!hasFocus()){
					Thread.sleep(1000);
				}
				Thread.sleep(delay);
				log("> run KeyPressThread");

				_typeKey(charCode, keyCode, alt, ctrl, shift);
			}catch(Exception e){
				log("Bad parameters passed to _typeKey");
				e.printStackTrace();
			}
			log("< run KeyPressThread");

		}
	}

	final private class KeyDownThread extends Thread{
		private int charCode;
		private int keyCode;
		private int delay;
		private Thread myPreviousThread = null;

		public KeyDownThread(int charCode, int keyCode, int delay, Thread myPreviousThread){
			log("KeyDownThread constructor " + charCode + ", " + keyCode);
			this.charCode = charCode;
			this.keyCode = keyCode;
			this.delay = delay;
			this.myPreviousThread = myPreviousThread;
		}

		public void run(){
			try{
				if(myPreviousThread != null)
					myPreviousThread.join();
				Thread.sleep(delay);
				log("> run KeyDownThread");
				while(!hasFocus()){
					Thread.sleep(1000);
				}
				int vkCode=getVKCode(charCode, keyCode);
				if(charCode >= 32){
					// if it is printable, then it lives in our hashmap
					KeyEvent event = (KeyEvent) charMap.get(new Integer(charCode));
					// see if we need to press shift to generate this
					// character
					if(event.isShiftDown()){
						robot.keyPress(KeyEvent.VK_SHIFT);
						shift=true;
					}
					if(event.isAltGraphDown()){
						robot.keyPress(KeyEvent.VK_ALT_GRAPH);
						altgraph=true;
					}
				}else{
					if(vkCode==KeyEvent.VK_ALT){
						alt=true;
					}else if(vkCode==KeyEvent.VK_CONTROL){
						ctrl=true;
					}else if(vkCode==KeyEvent.VK_SHIFT){
						shift=true;
					}else if(vkCode==KeyEvent.VK_ALT_GRAPH){
						altgraph=true;
					}
				}
				if(!isUnsafe(vkCode)){
					robot.keyPress(vkCode);
				}
			}catch(Exception e){
				log("Bad parameters passed to downKey");
				e.printStackTrace();
			}
			log("< run KeyDownThread");

		}
	}

	final private class KeyUpThread extends Thread{
		private int charCode;
		private int keyCode;
		private int delay;
		private Thread myPreviousThread = null;

		public KeyUpThread(int charCode, int keyCode, int delay, Thread myPreviousThread){
			log("KeyUpThread constructor " + charCode + ", " + keyCode);
			this.charCode = charCode;
			this.keyCode = keyCode;
			this.delay = delay;
			this.myPreviousThread = myPreviousThread;
		}

		public void run(){
			try{
				if(myPreviousThread != null)
					myPreviousThread.join();
				Thread.sleep(delay);
				log("> run KeyUpThread");
				while(!hasFocus()){
					Thread.sleep(1000);
				}
				int vkCode=getVKCode(charCode, keyCode);
				if(charCode >= 32){
					// if it is printable, then it lives in our hashmap
					KeyEvent event = (KeyEvent) charMap.get(new Integer(charCode));
					// see if we need to press shift to generate this
					// character
					if(event.isShiftDown()){
						robot.keyRelease(KeyEvent.VK_SHIFT);
						shift=false;
					}
					if(event.isAltGraphDown()){
						robot.keyRelease(KeyEvent.VK_ALT_GRAPH);
						altgraph=false;
					}
				}else{
					if(vkCode==KeyEvent.VK_ALT){
						alt=false;
					}else if(vkCode==KeyEvent.VK_CONTROL){
						ctrl=false;
					}else if(vkCode==KeyEvent.VK_SHIFT){
						shift=false;
					}else if(vkCode==KeyEvent.VK_ALT_GRAPH){
						altgraph=false;
					}
				}
				robot.keyRelease(vkCode);
			}catch(Exception e){
				log("Bad parameters passed to upKey");
				e.printStackTrace();
			}
			log("< run KeyUpThread");

		}
	}

	final private class MousePressThread extends Thread{
		private int mask;
		private int delay;
		private Thread myPreviousThread = null;

		public MousePressThread(int mask, int delay, Thread myPreviousThread){
			this.mask = mask;
			this.delay = delay;
			this.myPreviousThread = myPreviousThread;
		}

		public void run(){
			try{
				if(myPreviousThread != null)
					myPreviousThread.join();
				Thread.sleep(delay);
				log("> run MousePressThread");
				while(!hasFocus()){
					Thread.sleep(1000);
				}
				robot.mousePress(mask);
				robot.waitForIdle();
			}catch(Exception e){
				log("Bad parameters passed to mousePress");
				e.printStackTrace();
			}
			log("< run MousePressThread");

		}
	}

	final private class MouseReleaseThread extends Thread{
		private int mask;
		private int delay;
		private Thread myPreviousThread = null;

		public MouseReleaseThread(int mask, int delay, Thread myPreviousThread){
			this.mask = mask;
			this.delay = delay;
			this.myPreviousThread = myPreviousThread;
		}

		public void run(){
			try{
				if(myPreviousThread != null)
					myPreviousThread.join();
				Thread.sleep(delay);
				log("> run MouseReleaseThread ");
				while(!hasFocus()){
					Thread.sleep(1000);
				}
				robot.mouseRelease(mask);
				robot.waitForIdle();
			}catch(Exception e){
				log("Bad parameters passed to mouseRelease");
				e.printStackTrace();
			}

			log("< run MouseReleaseThread ");

		}
	}

	final private class MouseMoveThread extends Thread{
		private int x;
		private int y;
		private int delay;
		private int duration;
		private Thread myPreviousThread = null;

		public MouseMoveThread(int x, int y, int delay, int duration, Thread myPreviousThread){
			this.x = x;
			this.y = y;
			this.delay = delay;
			this.duration = duration;
			this.myPreviousThread = myPreviousThread;
		}

		public double easeInOutQuad(double t, double b, double c, double d){
			t /= d / 2;
			if(t < 1)
				return c / 2 * t * t + b;
			t--;
			return -c / 2 * (t * (t - 2) - 1) + b;
		};

		public void run(){
			try{
				if(myPreviousThread != null)
					myPreviousThread.join();
				Thread.sleep(delay);
				log("> run MouseMoveThread " + x + ", " + y);
				while(!hasFocus()){
					Thread.sleep(1000);
				}
				int x1 = lastMouseX;
				int x2 = x;
				int y1 = lastMouseY;
				int y2 = y;
				// shrink range by 1 px on both ends
				// manually move this 1px to trip DND code
				if(x1 != x2){
					int dx = x - lastMouseX;
					if(dx > 0){
						x1 += 1;
						x2 -= 1;
					}else{
						x1 -= 1;
						x2 += 1;
					}
				}
				if(y1 != y2){
					int dy = y - lastMouseY;
					if(dy > 0){
						y1 += 1;
						y2 -= 1;
					}else{
						y1 -= 1;
						y2 += 1;
					}

				}
				robot.setAutoDelay(Math.max(duration/100,1));
				robot.mouseMove(x1, y1);
				int d = 100;
				for (int t = 0; t <= d; t++){
					x1 = (int) easeInOutQuad((double) t, (double) lastMouseX,
							(double) x2 - lastMouseX, (double) d);
					y1 = (int) easeInOutQuad((double) t, (double) lastMouseY,
							(double) y2 - lastMouseY, (double) d);
					robot.mouseMove(x1, y1);
				}
				robot.mouseMove(x, y);
				lastMouseX = x;
				lastMouseY = y;
				robot.waitForIdle();
				robot.setAutoDelay(1);
			}catch(Exception e){
				log("Bad parameters passed to mouseMove");
				e.printStackTrace();
			}

			log("< run MouseMoveThread");

		}
	}

	final private class MouseWheelThread extends Thread{
		private int amount;
		private int delay;
		private int duration;
		private Thread myPreviousThread = null;

		public MouseWheelThread(int amount, int delay, int duration, Thread myPreviousThread){
			this.amount = amount;
			this.delay = delay;
			this.duration = duration;
			this.myPreviousThread = myPreviousThread;
		}

		public void run(){
			try{
				if(myPreviousThread != null)
					myPreviousThread.join();
				Thread.sleep(delay);
				log("> run MouseWheelThread " + amount);
				while(!hasFocus()){
					Thread.sleep(1000);
				}
				int dir = 1;
				if(System.getProperty("os.name").toUpperCase().indexOf("MAC") != -1){
					// yay for Apple
					dir = -1;
				}
				robot.setAutoDelay(Math.max(duration/Math.abs(amount),1));
				for(int i=0; i<Math.abs(amount); i++){
					robot.mouseWheel(amount>0?dir:-dir);
				}
				robot.setAutoDelay(1);
			}catch(Exception e){
				log("Bad parameters passed to mouseWheel");
				e.printStackTrace();
			}
			log("< run MouseWheelThread ");
		}
	}

	final private class RobotSecurityManager extends SecurityManager{
		// The applet's original security manager.
		// There is a bug in some people's Safaris that causes Safari to
		// basically hang on liveconnect calls.
		// Our security manager fixes it.

		private boolean isActive = false;
		private boolean isOpera = false;
		private SecurityManager oldsecurity = null;

		public RobotSecurityManager(boolean isOpera, SecurityManager oldsecurity){
			this.isOpera = isOpera;
			this.oldsecurity = oldsecurity;
		}

		public boolean checkTopLevelWindow(Object window){
			// If our users temporarily accept our cert for a session,
			// then use the same session to browse to a malicious website also using our applet,
			// that website can automatically execute the applet. 
			// To resolve this issue, RobotSecurityManager overrides checkTopLevelWindow
			// to check the JVM to see if there are other instances of the applet running on different domains.
			// If there are, it prompts the user to confirm that they want to run the applet before continuing. 

			// null is not supposed to be allowed
			// so we allow it to distinguish our security manager.
			if(window == null){
				isActive = !isActive;
				log("Active is now " + isActive);
			}
			return window == null ? true : oldsecurity
					.checkTopLevelWindow(window);
		}

		public void checkPermission(Permission p){
			// liveconnect SocketPermission resolve takes
			// FOREVER (like 6 seconds) in Safari
			// Java does like 50 of these on the first JS call
			// 6*50=300 seconds!
			// Opera freaks out though if we deny resolve
			if(isActive && !isOpera
					&& java.net.SocketPermission.class.isInstance(p)
					&& p.getActions().matches(".*resolve.*")){
				throw new SecurityException(
						"DOH: liveconnect resolve locks up Safari. Denying resolve request.");
			}else{
				oldsecurity.checkPermission(p);
			}
		}

		public void checkPermission(Permission perm, Object context){
			checkPermission(perm);
		}
	}

}
