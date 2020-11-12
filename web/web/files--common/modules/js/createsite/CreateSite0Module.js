

Wikijump.modules.CreateSite0Module = {};

Wikijump.modules.CreateSite0Module.listeners = {
	cancelClick: function(e){
		window.location.href="/";
	},

	nextClick: function(e){
		params = OZONE.utils.formToArray('new-site1');
		OZONE.ajax.requestModule("createsite/CreateSite1Module", params, Wikijump.modules.CreateSite0Module.callbacks.nextClick);

	},

	licenceSelect: function(e){
		val = document.getElementById("licence-select").value;
		if(val == 'other'){
			// show row
			document.getElementById('other-licence-row').style.display="";
		} else {
			document.getElementById('other-licence-row').style.display="none";
		}
	}
}
Wikijump.modules.CreateSite0Module.callbacks = {
	nextClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);

	}

}

YAHOO.util.Event.addListener("licence-select", "change", Wikijump.modules.CreateSite0Module.listeners.licenceSelect);

Wikijump.modules.CreateSite0Module.listeners.licenceSelect(null); // to initialize
