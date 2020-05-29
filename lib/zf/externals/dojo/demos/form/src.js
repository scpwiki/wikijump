dojo.require("dojo.parser");
dojo.require("dojo.data.ItemFileReadStore");

dojo.require("dijit.form.Form");
dojo.require("dijit.form.ValidationTextBox");
dojo.require("dijit.form.ComboBox");
dojo.require("dijit.form.FilteringSelect");
dojo.require("dijit.form.CheckBox");
dojo.require("dijit.form.DateTextBox");
dojo.require("dijit.form.CurrencyTextBox");
dojo.require("dijit.form.NumberSpinner");
dojo.require("dijit.form.Slider");
dojo.require("dijit.form.Textarea");
dojo.require("dijit.Editor");
dojo.require("dijit._editor.plugins.FontChoice");
dojo.require("dijit.form.Button");

// make dojo.toJson() print dates correctly (this feels a bit dirty)
Date.prototype.json = function(){ return dojo.date.stamp.toISOString(this, {selector: 'date'});};