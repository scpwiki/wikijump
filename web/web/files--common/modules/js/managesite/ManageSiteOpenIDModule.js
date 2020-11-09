

Wikijump.modules.ManageSiteOpenIDModule = {}

Wikijump.modules.ManageSiteOpenIDModule.vars = {
	randoms: new Object(),
	services: null
}

Wikijump.modules.ManageSiteOpenIDModule.listeners = {
	addEntry: function(e){
		var cont = $("sm-openid-templateform").innerHTML;
		var rand;
		do{
			rand = Math.ceil(Math.random()*10000)+1000;
		}while(Wikijump.modules.ManageSiteOpenIDModule.vars.randoms[rand] == true);
		Wikijump.modules.ManageSiteOpenIDModule.vars.randoms[rand] = true;

		cont = cont.replace(/RAND/g, rand);

		var div = document.createElement('div');
		div.id = 'sm-openid-entry-'+rand;
		div.innerHTML = cont;
		$("sm-openid-idblock").appendChild(div);

		var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
		myDataSource.scriptQueryParam="q";
		myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";

		var myAutoComp = new YAHOO.widget.AutoComplete("sm-openid-p-"+rand,"sm-openid-p-list-"+rand, myDataSource);
		myAutoComp.formatResult = function(aResultItem, sQuery) {
			var title = aResultItem[1];
			var unixName = aResultItem[0];
			if(unixName!= null){
				return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 80%;">('+title+')</div>';
			} else {
				return "";
			}
		}

		myAutoComp.forceSelection = true;
		myAutoComp.autoHighlight = true;
		myAutoComp.minQueryLength = 2;
		myAutoComp.queryDelay = 0.5;

		OZONE.visuals.scrollTo(div);

	},

	deleteEntry: function(e, id){
		var el = $('sm-openid-entry-'+id);
		if(el){
			el.parentNode.removeChild(el);
		}

	},

	onIdentityChange: function(e, id){
		var url = $("sm-openid-urlid-"+id).value;
		var serverInput = $("sm-openid-urlserver-"+id);

		// check if URL matches any of the patterns...
		var os = Wikijump.modules.ManageSiteOpenIDModule.vars.services;
		var pattern = null;
		var server = null;
		var reg = null;

		for(var i in os){
			pattern = os[i]['pattern'];
			reg = new RegExp(pattern);
			if(reg.test(url)){
				$("sm-openid-urlserver-"+id).value = os[i]['server'];
			}
		}

	},

	save: function(e){
		var p = new Object();
		p.action = "ManageSiteOpenIDAction";
		p.event = "saveOpenID";
		p.enableOpenID = $("sm-openid-enable").checked;

		var vals = new Array();
		vals[0] = OZONE.utils.formToArray($("sm-openid-form-0"));

		var forms = $("sm-openid-idblock").getElementsByTagName("form");
		for(var i=0;i<forms.length;i++){
			vals.push(OZONE.utils.formToArray(forms[i]));
		}

		p.vals = JSON.stringify(vals);
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteOpenIDModule.callbacks.save);

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}

}

Wikijump.modules.ManageSiteOpenIDModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
}
Wikijump.modules.ManageSiteOpenIDModule.init = function(){
	var os = $("sm-openid-patterns").innerHTML;

	Wikijump.modules.ManageSiteOpenIDModule.vars.services = JSON.parse(os);
}

Wikijump.modules.ManageSiteOpenIDModule.init();
