dojo.provide("dojox.form.DropDownSelect");

dojo.require("dojox.form._FormSelectWidget");
dojo.require("dojox.form._HasDropDown");
dojo.require("dijit.Menu");

dojo.requireLocalization("dijit.form", "validate");

dojo.declare("dojox.form.DropDownSelect", [dojox.form._FormSelectWidget, dojox.form._HasDropDown], {
	// summary:
	//		This is a "Styleable" select box - it is basically a DropDownButton which
	//		can take as its input a <select>.

	baseClass: "dojoxDropDownSelect",
	
	templatePath: dojo.moduleUrl("dojox.form", "resources/DropDownSelect.html"),
	
	// attributeMap: Object
	//		Add in our style to be applied to the focus node
	attributeMap: dojo.mixin(dojo.clone(dojox.form._FormSelectWidget.prototype.attributeMap),{style:"tableNode"}),
	
	// required: Boolean
	//		Can be true or false, default is false.
	required: false,

	// state: String
	//		Shows current state (ie, validation result) of input (Normal, Warning, or Error)
	state: "",

	//	tooltipPosition: String[]
	//		See description of dijit.Tooltip.defaultPosition for details on this parameter.
	tooltipPosition: [],

	// emptyLabel: string
	//		What to display in an "empty" dropdown
	emptyLabel: "",
	
	// _isLoaded: boolean
	//		Whether or not we have been loaded
	_isLoaded: false,
	
	// _childrenLoaded: boolean
	//		Whether or not our children have been loaded
	_childrenLoaded: false,
	
	_fillContent: function(){
		// summary:  
		//		Set the value to be the first, or the selected index
		this.inherited(arguments);
		if(this.options.length && !this.value){
			var si = this.srcNodeRef.selectedIndex;
			this.value = this.options[si != -1 ? si : 0].value;
		}
		
		// Create the dropDown widget
		this.dropDown = new dijit.Menu();
		dojo.addClass(this.dropDown.domNode, this.baseClass + "Menu");
	},

	_getMenuItemForOption: function(/* dojox.form.__SelectOption */ option){
		// summary:
		//		For the given option, return the menu item that should be
		//		used to display it.  This can be overridden as needed
		if(!option.value){
			// We are a separator (no label set for it)
			return new dijit.MenuSeparator();
		}else{
			// Just a regular menu option
			var click = dojo.hitch(this, "_setValueAttr", option);
			return new dijit.MenuItem({
				option: option,
				label: option.label,
				onClick: click,
				disabled: option.disabled || false
			});
		}
	},

	_addOptionItem: function(/* dojox.form.__SelectOption */ option){
		// summary:
		//		For the given option, add a option to our dropdown
		//		If the option doesn't have a value, then a separator is added 
		//		in that place.
		this.dropDown.addChild(this._getMenuItemForOption(option));
	},

	_getChildren: function(){ return this.dropDown.getChildren(); },
	
	_loadChildren: function(){
		// summary: 
		//		Resets the menu and the length attribute of the button - and
		//		ensures that the label is appropriately set.
		this.inherited(arguments);
		var len = this.options.length;
		this._isLoaded = false;
		this._childrenLoaded = true;
		
		// Set our length attribute and our value
		if(!this._iReadOnly){
			this.attr("readOnly", (len === 1));
			delete this._iReadOnly;
		}
		if(!this._iDisabled){
			this.attr("disabled", (len === 0));
			delete this._iDisabled;
		}
		this._setValueAttr(this.value);
	},
	
	_setDisplay: function(/*String*/ newDisplay){
		// summary: sets the display for the given value (or values)
		this.containerNode.innerHTML = '<span class=" ' + this.baseClass + 'Label">' +
					(newDisplay || this.emptyLabel || "&nbsp;") +
					'</span>';
		this._layoutHack();
	},

	validate: function(/*Boolean*/ isFocused){
		// summary:
		//		Called by oninit, onblur, and onkeypress.
		// description:
		//		Show missing or invalid messages if appropriate, and highlight textbox field.
		var isValid = this.isValid(isFocused);
		this.state = isValid ? "" : "Error";
		this._setStateClass();
		dijit.setWaiState(this.focusNode, "invalid", isValid ? "false" : "true");
		var message = isValid ? "" : this._missingMsg;
		if(this._message !== message){
			this._message = message;
			dijit.hideTooltip(this.domNode);
			if(message){
				dijit.showTooltip(message, this.domNode, this.tooltipPosition);
			}
		}
		return isValid;		
	},

	isValid: function(/*Boolean*/ isFocused){
		// summary: Whether or not this is a valid value
		return (!this.required || !(/^\s*$/.test(this.value)));
	},
	
	reset: function(){
		// summary: Overridden so that the state will be cleared.
		this.inherited(arguments);
		dijit.hideTooltip(this.domNode);
		this.state = "";
		this._setStateClass();
		delete this._message;
	},

	postMixInProperties: function(){
		// summary: set the missing message
		this.inherited(arguments);
		this._missingMsg = dojo.i18n.getLocalization("dijit.form", "validate", 
									this.lang).missingMessage;
	},
	
	postCreate: function(){
		this.inherited(arguments);
		if(dojo.attr(this.srcNodeRef, "disabled")){
			this.attr("disabled", true);
		}
	},

	startup: function(){
		if(this._started){ return; }

		// the child widget from srcNodeRef is the dropdown widget.  Insert it in the page DOM,
		// make it invisible, and store a reference to pass to the popup code.
		if(!this.dropDown){
			var dropDownNode = dojo.query("[widgetId]", this.dropDownContainer)[0];
			this.dropDown = dijit.byNode(dropDownNode);
			delete this.dropDownContainer;
		}
		this.inherited(arguments);
	},
	
	_onMenuMouseup: function(e){
		// override this to actually "pretend" to do a click on our menu - it will
		// call onExecute - so it will close our popup for us.  For non-menu
		// popups, it will not execute.
		var dropDown = this.dropDown, t = e.target;
		
		if(dropDown.onItemClick){
			var menuItem;
			while(t && !(menuItem = dijit.byNode(t))){
				t = t.parentNode;
			}
			if(menuItem && menuItem.onClick && menuItem.getParent){
				menuItem.getParent().onItemClick(menuItem, e);
			}
		}
		// TODO: how to handle non-menu popups?
	},

	isLoaded: function(){
		return this._isLoaded;
	},
	
	loadDropDown: function(/* Function */ loadCallback){
		// summary: populates the menu
		this._loadChildren();
		this._isLoaded = true;
		loadCallback();
	},
	
	_setReadOnlyAttr: function(value){
		this._iReadOnly = value;
		if(!value && this._childrenLoaded && this.options.length === 1){
			return;
		}
		this.readOnly = value;
	},
	
	_setDisabledAttr: function(value){
		this._iDisabled = value;
		if(!value && this._childrenLoaded && this.options.length === 0){
			return;
		}
		this.inherited(arguments);
	}
});
