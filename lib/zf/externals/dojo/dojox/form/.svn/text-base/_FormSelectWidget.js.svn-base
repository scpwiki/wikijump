dojo.provide("dojox.form._FormSelectWidget");

dojo.require("dijit.form._FormWidget");

dojo.declare("dojox.form._FormSelectWidget", dijit.form._FormValueWidget, {
	// multiple: Boolean
	//		Matches the select's "multiple=" value
	multiple: "",
	
	// _multiValue: Boolean
	//		Whether or not we are multi-valued (for form)
	_multiValue: false,

	/*=====
	dojox.form.__SelectOption = function(){
		//	value: String
		//		The value of the option.  Setting to empty (or missing) will
		//		place a separator at that location
		//	label: String
		//		The label for our option.  It can contain html tags.
		//  selected: Boolean
		//		Whether or not we are a selected option
		//	disabled: Boolean
		//		Whether or not this specific option is disabled
		this.value = value;
		this.label = label;
		this.selected = selected;
		this.disabled = disabled;
	}
	=====*/

	// options: dojox.form.__SelectOption[]
	//		The set of options for our select item.  Roughly corresponds to
	//      the html <option> tag.
	options: null,
	
	getOptions: function(/* anything */ valueOrIdx){
		// summary:
		//		Returns a given option (or options).
		// valueOrIdx:
		//		If passed in as a string, that string is used to look up the option
		//		in the array of options - based on the value property. 
		//		(See dojox.form.__SelectOption).
		//
		//		If passed in a number, then the option with the given index (0-based)
		//		within this select will be returned.
		//		
		//		If passed in a dojox.form.__SelectOption, the same option will be
		//		returned if and only if it exists within this select.
		//		
		//		If passed an array, then an array will be returned with each element
		//		in the array being looked up.
		//
		//		If not passed a value, then all options will be returned
		//
		// returns:
		//		The option corresponding with the given value or index.  null
		//		is returned if any of the following are true:
		//			- A string value is passed in which doesn't exist
		//			- An index is passed in which is outside the bounds of the array of options
		//			- A dojox.form.__SelectOption is passed in which is not a part of the select
		
		// NOTE: the compare for passing in a dojox.form.__SelectOption checks
		//		if the value property matches - NOT if the exact option exists
		// NOTE: if passing in an array, null elements will be placed in the returned
		//		array when a value is not found.
		var lookupValue = valueOrIdx, opts = this.options || [], l = opts.length;

		if(lookupValue === undefined){
			return opts; // dojox.form.__SelectOption[]
		}
		if(dojo.isArray(lookupValue)){
			return dojo.map(lookupValue, "return this.getOptions(item);", this); // dojox.form.__SelectOption[]
		}
		if(dojo.isObject(valueOrIdx)){
			// We were passed an option - so see if it's in our array (directly),
			// and if it's not, try and find it by value.
			if (!dojo.some(this.options, function(o, idx){
				if (o === lookupValue ||
					(o.value && o.value === lookupValue.value)){
					lookupValue = idx;
					return true;
				}
				return false;
			})){
				lookupValue = -1;
			}
		}
		if(typeof lookupValue == "string"){
			for(var i=0; i<l; i++){
				if(opts[i].value === lookupValue){
					lookupValue = i;
					break;
				}
			}
		}
		if(typeof lookupValue == "number" && lookupValue >= 0 && lookupValue < l){
			return this.options[lookupValue] // dojox.form.__SelectOption
		}
		return null; // null
	},
	
	addOption: function(/* dojox.form.__SelectOption, dojox.form.__SelectOption[] */ option){
		//	summary:
		//		Adds an option or options to the end of the select.  If value
		//		of the option is empty or missing, a separator is created instead.
		//		Passing in an array of options will yeild slightly better performance
		//		since the children are only loaded once.
		if(!dojo.isArray(option)){ option = [option]; }
		dojo.forEach(option, function(i){
			if(i && dojo.isObject(i)){
				this.options.push(i);
			}
		}, this);
		this._loadChildren();
	},
	
	removeOption: function(/* string, dojox.form.__SelectOption, number, or array */ valueOrIdx){
		// summary:
		//		Removes the given option or options.  You can remove by string 
		//		(in which case the value is removed), number (in which case the
		//		index in the options array is removed), or select option (in 
		//		which case, the select option with a matching value is removed).
		//		You can also pass in an array of those values for a slightly
		//		better performance since the children are only loaded once.
		if(!dojo.isArray(valueOrIdx)){ valueOrIdx = [valueOrIdx]; }
		var oldOpts = this.getOptions(valueOrIdx);
		dojo.forEach(oldOpts, function(i){
			this.options = dojo.filter(this.options, function(node, idx){
				return (node.value !== i.value);
			});
			this._removeOptionItem(i);
		}, this);
		this._loadChildren();
	},
	
	updateOption: function(/* dojox.form.__SelectOption, dojox.form.__SelectOption[] */ newOption){
		// summary:
		//		Updates the values of the given option.  The option to update
		//		is matched based on the value of the entered option.  Passing
		//		in an array of new options will yeild better performance since
		//		the children will only be loaded once.
		if(!dojo.isArray(newOption)){ newOption = [newOption]; }
		dojo.forEach(newOption, function(i){
			var oldOpt = this.getOptions(i), k;
			if(oldOpt){
				for(k in i){ oldOpt[k] = i[k]; }
			}
		}, this);
		this._loadChildren();
	},

	_setValueAttr: function(/*anything*/ newValue, /*Boolean, optional*/ priorityChange){
		// summary: set the value of the widget.
		// If a string is passed, then we set our value from looking it up.
		var opts = this.getOptions() || [];
		if(!dojo.isArray(newValue)){
			newValue = [newValue];
		}
		dojo.forEach(newValue, function(i, idx){
			if(!dojo.isObject(i)){
				i = i + "";
			}
			if(typeof i === "string"){
				newValue[idx] = dojo.filter(opts, function(node){
					return node.value === i;
				})[0] || {value: "", label: ""};
			}
		}, this);
		
		// Make sure some sane default is set
		newValue = dojo.filter(newValue, function(i){ return i && i.value; });
		if(!this._multiValue && (!newValue[0] || !newValue[0].value) && opts.length){
			newValue[0] = opts[0];
		}
		dojo.forEach(opts, function(i){
			i.selected = dojo.some(newValue, function(v){ return v.value === i.value; });
		});
		var val = dojo.map(newValue, function(i){ return i.value; }),
			disp = dojo.map(newValue, function(i){ return i.label; });
		
		this.value = this._multiValue ? val : val[0];
		this._setDisplay(this._multiValue ? disp : disp[0]);
		this._updateSelection();
		this._handleOnChange(this.value, priorityChange);
	},
	
	_getValueDeprecated: false, // remove when _FormWidget:getValue is removed
	getValue: function(){
		// summary: get the value of the widget.
		return this._lastValue;
	},

	undo: function(){
		// summary: restore the value to the last value passed to onChange
		this._setValueAttr(this._lastValueReported, false);
	},

	_loadChildren: function(){
		// summary: 
		//		Loads the children represented by this widget's optiosn.
		// reset the menu to make it "populatable on the next click
		dojo.forEach(this._getChildren(), function(child){
			child.destroyRecursive();
		});	
		// Add each menu item
		dojo.forEach(this.options, this._addOptionItem, this);
		
		// Update states
		this._updateSelection();
	},

	_updateSelection: function(){
		// summary:
		//		Sets the "selected" class on the item for styling purposes
		this.value = this._getValueFromOpts();
		var val = this.value;
		if(!dojo.isArray(val)){
			val = [val];
		}
		if(val && val[0]){
			dojo.forEach(this._getChildren(), function(child){
				var isSelected = dojo.some(val, function(v){
					return child.option && (v === child.option.value);
				});
				dojo.toggleClass(child.domNode, this.baseClass + "SelectedOption", isSelected);
				dijit.setWaiState(child.domNode, "selected", isSelected);
			}, this);
		}
		this._handleOnChange(this.value);
	},
	
	_getValueFromOpts: function(){
		// summary:
		//		Returns the value of the widget by reading the options for
		//		the selected flag
		var opts = this.getOptions() || [];
		if(!this._multiValue && opts.length){
			// Mirror what a select does - choose the first one
			var opt = dojo.filter(opts, function(i){
				return i.selected;
			})[0];
			if(opt && opt.value){
				return opt.value
			}else{
				opts[0].selected = true;
				return opts[0].value;
			}
		}else if(this._multiValue){
			// Set value to be the sum of all selected
			return dojo.map(dojo.filter(opts, function(i){
				return i.selected;
			}), function(i){
				return i.value;
			}) || [];
		}
		return "";
	},
	
	postMixInProperties: function(){
		this._multiValue = (this.multiple.toLowerCase() === "true");
		this.inherited(arguments);
	},
	
	_fillContent: function(){
		// summary:  
		//		Loads our options and sets up our dropdown correctly.  We 
		//		don't want any content, so we don't call any inherit chain
		//		function.
		var opts = this.options;
		if(!opts){
			opts = this.options = this.srcNodeRef ? dojo.query(">", 
						this.srcNodeRef).map(function(node){
							if(node.getAttribute("type") === "separator"){
								return { value: "", label: "", selected: false, disabled: false };
							}
							return { value: node.getAttribute("value"),
										label: String(node.innerHTML),
										selected: node.getAttribute("selected") || false,
										disabled: node.getAttribute("disabled") || false };
						}, this) : [];
		}
		if(!this.value){
			this.value = this._getValueFromOpts();
		}else if(this._multiValue && typeof this.value == "string"){
			this.value = this.value.split(",");
		}
	},

	postCreate: function(){
		// summary: sets up our event handling that we need for functioning
		//			as a select
		dojo.setSelectable(this.focusNode, false);
		this.inherited(arguments);

		// Make our event connections for updating state
		this.connect(this, "onChange", "_updateSelection");
		this.connect(this, "startup", "_loadChildren");
		
		this._setValueAttr(this.value, null);
	},
	
	_addOptionItem: function(/* dojox.form.__SelectOption */ option){
		// summary:
		//		User-overridable function which, for the given option, adds an 
		//		item to the select.  If the option doesn't have a value, then a 
		//		separator is added in that place.  Make sure to store the option
		//		in the created option widget.
	},
	
	_removeOptionItem: function(/* dojox.form.__SelectOption */ option){
		// summary:
		//		User-overridable function which, for the given option, removes
		//		its item from the select.
	},
	
	_setDisplay: function(/*String or String[]*/ newDisplay){
		// summary: Overridable function which will set the display for the 
		//			widget.  newDisplay is either a string (in the case of 
		//			single selects) or array of strings (in the case of multi-
		//			selects)
	},
	
	_getChildren: function(){
		// summary: Overridable function to return the children that this widget
		//			contains.
		return [];
	},
	
	_getSelectedOptionsAttr: function(){
		// summary: hooks into this.attr to provide a mechanism for getting the
		//			option items for the current value of the widget.
		return this.getOptions(this.attr("value"));
	}

});
