dojo.provide("dijit.form.MultiSelect");

dojo.require("dijit.form._FormWidget");

dojo.declare("dijit.form.MultiSelect",dijit.form._FormWidget,{
	// summary: Wrapper for a native select multiple="true" element to
	//		interact with dijit.form.Form

	// size: Number
	//		Number of elements to display on a page
	//		NOTE: may be removed in version 2.0, since elements may have variable height;
	//		set the size via style="..." or CSS class names instead.
	size: 7,
	
	templateString: "<select multiple='true' name='${name}' dojoAttachPoint='containerNode,focusNode' dojoAttachEvent='onchange: _onChange'></select>",

	attributeMap: dojo.mixin(dojo.clone(dijit.form._FormWidget.prototype.attributeMap),
		{size:"focusNode"}),

	reset: function(){
		// TODO: once we inherit from FormValueWidget this won't be needed
		this._hasBeenBlurred = false;
		this._setValueAttr(this._resetValue, true);
	},

	addSelected: function(/* dijit.form.MultiSelect */select){
		// summary: Move the selected nodes af an passed Select widget
		//			instance to this Select widget.
		//
		// example:
		// |	// move all the selected values from "bar" to "foo"
		// | 	dijit.byId("foo").addSelected(dijit.byId("bar"));
		
		select.getSelected().forEach(function(n){
			this.containerNode.appendChild(n);
			if(dojo.isIE){ // tweak the node to force IE to refresh (aka _layoutHack on FF2)
				var s = dojo.getComputedStyle(n);
				if(s){
					var filter = s.filter;
					n.style.filter = "alpha(opacity=99)";
					n.style.filter = filter;
				}
			}
			// scroll to bottom to see item
			// cannot use scrollIntoView since <option> tags don't support all attributes
			// does not work on IE due to a bug where <select> always shows scrollTop = 0
			this.domNode.scrollTop = this.domNode.offsetHeight; // overshoot will be ignored
			// scrolling the source select is trickier esp. on safari who forgets to change the scrollbar size
			var oldscroll = select.domNode.scrollTop;
			select.domNode.scrollTop = 0;
			select.domNode.scrollTop = oldscroll;
		},this);
	},
					
	getSelected: function(){
		// summary: Access the NodeList of the selected options directly
		return dojo.query("option",this.containerNode).filter(function(n){
			return n.selected; // Boolean
		});
	},
	
	_getValueAttr: function(){
		// summary:
		//		Hook so attr('value') works.
		// description:
		//		Returns an array of the selected options' values.
		return this.getSelected().map(function(n){
			return n.value;
		});
	},
	
	_multiValue: true, // for Form

	_setValueAttr: function(/* Array */values){
		// summary:
		//		Hook so attr('value', values) works.
		// description:
		//		Set the value(s) of this Select based on passed values
		dojo.query("option",this.containerNode).forEach(function(n){
			n.selected = (dojo.indexOf(values,n.value) != -1);
		});
	},
		
	invertSelection: function(onChange){
		// summary: Invert the selection
		// onChange: Boolean
		//		If null, onChange is not fired.
		dojo.query("option",this.containerNode).forEach(function(n){
			n.selected = !n.selected;
		});
		this._handleOnChange(this.attr('value'), onChange==true);
	},

	_onChange: function(/*Event*/ e){
		this._handleOnChange(this.attr('value'), true);
	},
	
	// for layout widgets:
	resize: function(/* Object */size){
		if(size){
			dojo.marginBox(this.domNode, size);
		}
	},
	
	postCreate: function(){
		this._onChange();
	}
});
