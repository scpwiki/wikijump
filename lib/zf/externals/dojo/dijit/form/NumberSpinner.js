dojo.provide("dijit.form.NumberSpinner");

dojo.require("dijit.form._Spinner");
dojo.require("dijit.form.NumberTextBox");

dojo.declare(
"dijit.form.NumberSpinner",
[dijit.form._Spinner, dijit.form.NumberTextBoxMixin],
{
	// summary:
	// extends NumberTextBox to add up/down arrows and pageup/pagedown for incremental change to the value

	required: true,

	adjust: function(/* Object */ val, /*Number*/ delta){
		// summary: change Number val by the given amount
		if(isNaN(val) && delta != 0){ // blank or invalid value and they want to spin, so create defaults
			var increasing = (delta > 0),
				gotMax = (typeof this.constraints.max == "number"),
				gotMin = (typeof this.constraints.min == "number");
			val = increasing? (gotMin? this.constraints.min : (gotMax? this.constraints.max : 0)) :
					(gotMax? this.constraints.max : (gotMin? this.constraints.min : 0));
		}
		var newval = val+delta;
		if(isNaN(val) || isNaN(newval)){ return val; }
		if((typeof this.constraints.max == "number") && (newval > this.constraints.max)){
			newval = this.constraints.max;
		}
		if((typeof this.constraints.min == "number") && (newval < this.constraints.min)){
			newval = this.constraints.min;
		}
		return newval;
	},
	_onKeyPress: function(e){
		if((e.charOrCode == dojo.keys.HOME || e.charOrCode == dojo.keys.END) && !e.ctrlKey && !e.altKey){
			var value = e.charOrCode == dojo.keys.HOME ? this.constraints["min"] : this.constraints["max"];
			if (value){
				this._setValueAttr(value,true);
			}
			// eat home or end key whether we change the value or not
			dojo.stopEvent(e);
			return false;
		}
		else{
			return this.inherited(arguments);
		}
	}
});
