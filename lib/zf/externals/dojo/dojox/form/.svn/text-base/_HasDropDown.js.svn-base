dojo.provide("dojox.form._HasDropDown");

dojo.require("dijit._Widget");

dojo.declare("dojox.form._HasDropDown",
	null,
	{
		//	summary:
		//		Mixin for widgets that need drop down ability.
		//
		//	dropDownNode: DomNode
		//		responds to click events.  "dropDownNode" can be set via
		//		a dojoAttachPoint assignment.  If missing, then either focusNode
		//		or domNode (if focusNode is also missing) will be used.
		dropDownNode: null,
		
		//	popupStateNode: DomNode
		//		the node to set the popupActive class on.  If missing, then 
		//		focusNode or dropDownNode (if focusNode is missing) will be used.
		popupStateNode: null,
		
		//	aroundNode: DomNode
		//		the node to display the popup around.  If missing, then 
		//		domNode will be used.
		aroundNode: null,
		
		//	dropDown: Widget
		//		the widget to display in the popup.  This widget *must* be 
		//		defined before the startup function is called.
		dropDown: null,
		
		//	autoWidth: Boolean
		//		Set to true to make the drop down at least as wide as this 
		//		widget.  Set to false if the drop down should just be its
		//		default width
		autoWidth: true,
		
		//	_stopClickEvents: Boolean
		//		When set to false, the click events will not be stopped, in
		//		case you want to use them in your subwidget
		_stopClickEvents: true,
		
		_onMenuMouseup: function(/*Event*/ e){
			// summary: called with the mouseup event if the mouseup occurred
			//			over the menu.  You can try and use this event in
			//			order to automatically execute your dropdown (as
			//			if it were clicked).  You mightwant to close your menu 
			//			as a part of this function.
		},
		
		_onDropDownMouse: function(/*Event*/ e){
			// summary: callback when the user mouse clicks on menu popup node
			//
			// We handle mouse events using onmousedown in order to allow for
			// selecting via a drag.  So, our click is already handled, unless
			// we are executed via keypress - in which case, this._seenKeydown
			// will be set to true.			
			if(e.type == "click" && !this._seenKeydown){ return; }
			this._seenKeydown = false;
			
			// If we are a mouse event, set up the mouseup handler
			if(e.type == "mousedown"){
				this._docHandler = this.connect(dojo.doc, "onmouseup", "_onDropDownMouseup");
			}
			if(this.disabled || this.readOnly){ return; }
			if(this._stopClickEvents){
				dojo.stopEvent(e);
			}
			this.toggleDropDown();

			// If we are a click, then we'll pretend we did a mouse up
			if(e.type == "click" || e.type == "keypress"){
				this._onDropDownMouseup();
			}
		},

		_onDropDownMouseup: function(/*Event?*/ e){
			// summary: callback when the user lifts their mouse - if we are
			//			over the menu, we execute it, otherwise, we focus our 
			//			dropDown node.  If the event is missing, then we are not
			//			a mouseup event.
			if(e && this._docHandler){
				this.disconnect(this._docHandler);
			}
			var dropDown = this.dropDown, overMenu = false;
			
			if(e && this._opened){
				// Find out if our target is somewhere in our dropdown widget
				var t = e.target;
				while(t && !overMenu){
					if(dojo.hasClass(t, "dijitPopup")){
						overMenu = true;
					}else{
						t = t.parentNode;
					}
				}
				if(overMenu){
					this._onMenuMouseup(e);
					return;
				}
			}
			if(this._opened && dropDown.focus){
				// delay so that we don't steal our own focus.
				window.setTimeout(dojo.hitch(dropDown, "focus"), 1);
			}else{
				dijit.focus(this.focusNode);
			}
		},
		
		_setupDropdown: function(){
			//	summary:
			//		set up nodes and connect our mouse and keypress events
			this.dropDownNode = this.dropDownNode || this.focusNode || this.domNode;
			this.popupStateNode = this.popupStateNode || this.focusNode || this.dropDownNode;
			this.aroundNode = this.aroundNode || this.domNode;
			this.connect(this.dropDownNode, "onmousedown", "_onDropDownMouse");
			this.connect(this.dropDownNode, "onclick", "_onDropDownMouse");
			this.connect(this.dropDownNode, "onkeydown", "_onDropDownKeydown");
			this.connect(this.dropDownNode, "onblur", "_onDropDownBlur");
			this.connect(this.dropDownNode, "onkeypress", "_onKey");	
			
			// If we have a _setStateClass function (which happens when
			// we are a form widget), then we need to connect our open/close
			// functions to it
			if(this._setStateClass){
				this.connect(this, "openDropDown", "_setStateClass");
				this.connect(this, "closeDropDown", "_setStateClass");
			}
		},
		
		postCreate: function(){
			this._setupDropdown();
			this.inherited("postCreate", arguments);
		},
		
		startup: function(){
			dijit.popup.prepare(this.dropDown.domNode);
			this.inherited("startup", arguments);
		},
		
		destroyDescendants: function(){
			if(this.dropDown){
				this.dropDown.destroyRecursive();
				delete this.dropDown;
			}
			this.inherited("destroyDescendants", arguments);
		},

		_onDropDownKeydown: function(/*Event*/ e){
			this._seenKeydown = true;
		},
		
		_onKeyPress: function(/*Event*/ e){
			if(this._opened && e.charOrCode == dojo.keys.ESCAPE && !e.shiftKey && !e.ctrlKey && !e.altKey){
				this.toggleDropDown();
				dojo.stopEvent(e);
				return;
			}
			this.inherited(arguments);
		},

		_onDropDownBlur: function(/*Event*/ e){
			this._seenKeydown = false;
		},

		_onKey: function(/*Event*/ e){
			// summary: callback when the user presses a key on menu popup node
			if(this.disabled || this.readOnly){ return; }
			var d = this.dropDown;
			if(d && this._opened && d.handleKey){
				if(d.handleKey(e) === false){ return; }
			}
			if(d && this._opened && e.keyCode == dojo.keys.ESCAPE){
				this.toggleDropDown();
				return;
			}
			if(e.keyCode == dojo.keys.DOWN_ARROW){
				this._onDropDownMouse(e);
			}
		},

		_onBlur: function(){
			// summary: called magically when focus has shifted away from this widget and it's dropdown
			this.closeDropDown();
			// don't focus on button.  the user has explicitly focused on something else.
			this.inherited("_onBlur", arguments);
		},
		
		isLoaded: function(){
			// summary: returns whether or not the dropdown is loaded.  This can
			//		be overridden in order to force a call to loadDropDown().
			return true;
		},
		
		loadDropDown: function(/* Function */ loadCallback){
			// summary: loads the data for the dropdown, and at some point, calls
			//		the given callback
			loadCallback();
		},

		toggleDropDown: function(){
			// summary: toggle the drop-down widget; if it is up, close it, if not, open it
			if(this.disabled || this.readOnly){ return; }
			this.focus();
			var dropDown = this.dropDown;
			if(!dropDown){ return; }
			if(!this._opened){
				// If we aren't loaded, load it first so there isn't a flicker
				if(!this.isLoaded()){
					this.loadDropDown(dojo.hitch(this, "openDropDown"));
					return;
				}else{
					this.openDropDown();
				}
			}else{
				this.closeDropDown();
			}
		},
		
		openDropDown: function(){
			// summary: opens the dropdown for this widget - it returns the 
			//			return value of dijit.popup.open
			var dropDown = this.dropDown;
			var oldWidth=dropDown.domNode.style.width;
			var self = this;

			var retVal = dijit.popup.open({
				parent: this,
				popup: dropDown,
				around: this.aroundNode,
				orient:
					// TODO: add user-defined positioning option, like in Tooltip.js
					this.isLeftToRight() ? {'BL':'TL', 'BR':'TR', 'TL':'BL', 'TR':'BR'}
					: {'BR':'TR', 'BL':'TL', 'TR':'BR', 'TL':'BL'},
				onExecute: function(){
					self.closeDropDown(true);
				},
				onCancel: function(){
					self.closeDropDown(true);
				},
				onClose: function(){
					dropDown.domNode.style.width = oldWidth;
					dojo.attr(self.popupStateNode, "popupActive", false);
					dojo.removeClass(self.popupStateNode, "dojoxHasDropDownOpen");
					self._opened = false;
					self.state = "";
				}
			});
			if(this.autoWidth && this.domNode.offsetWidth > dropDown.domNode.offsetWidth){
				var adjustNode = null;
				if(!this.isLeftToRight()){
					adjustNode = dropDown.domNode.parentNode;
					var oldRight = adjustNode.offsetLeft + adjustNode.offsetWidth;
				}
				// make menu at least as wide as the node
				if(dropDown.resize){
					dropDown.resize({w: this.domNode.offsetWidth});
				}else{
					dojo.marginBox(dropDown.domNode, {w: this.domNode.offsetWidth});
				}
				if(adjustNode){
					adjustNode.style.left = oldRight - this.domNode.offsetWidth + "px";
				}
			}
			dojo.attr(this.popupStateNode, "popupActive", "true");
			dojo.addClass(self.popupStateNode, "dojoxHasDropDownOpen");
			this._opened=true;
			this.state="Opened";
			if(dropDown.focus){
				dropDown.focus();
			}
			// TODO: set this.checked and call setStateClass(), to affect button look while drop down is shown
			return retVal;
		},
	
		closeDropDown: function(/*Boolean*/ focus){
			// summary: Closes the drop down on this widget
			if(this._opened){
				dijit.popup.close(this.dropDown);
				if(focus){ this.focus(); }
				this._opened = false;
				this.state = "";
			}
		}
		
	}
);