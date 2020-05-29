dojo.provide("dijit._editor.plugins.TabIndent");
dojo.experimental("dijit._editor.plugins.TabIndent");

dojo.require("dijit._editor._Plugin");

dojo.declare("dijit._editor.plugins.TabIndent",
	dijit._editor._Plugin,
	{
		//summary: This plugin is used to allow the use of the tab and shift-tab keys
		//		   to indent/outdent list items.  This overrides the default behavior
		//         of moving focus from/to the toolbar
		
		useDefaultCommand: false,
		buttonClass: dijit.form.ToggleButton,
		command: "tabIndent",

		_initButton: function(){
			this.inherited("_initButton", arguments);
				this.connect(this.button, "onClick", this._tabIndent);		
			},

		updateState: function(){
			var _e = this.editor;
			var _c = this.command;
			if(!_e){ return; }
			if(!_e.isLoaded){ return; }
			if(!_c.length){ return; }
			if(this.button){
				try{
					var enabled = _e.isTabIndent;
					if(typeof this.button.checked == 'boolean'){ 
						this.button.attr('checked', enabled);
					}
				}catch(e){
					console.debug(e);
				}
			}
		},

		_tabIndent: function(){
			// toggle the value for isTabIndent
			this.editor.isTabIndent = !this.editor.isTabIndent;
		}
	}
);

dojo.subscribe(dijit._scopeName + ".Editor.getPlugin",null,function(o){
	if(o.plugin){ return; }
	switch(o.args.name){
	case "tabIndent":
		o.plugin = new dijit._editor.plugins.TabIndent({command: o.args.name});
	}
});
