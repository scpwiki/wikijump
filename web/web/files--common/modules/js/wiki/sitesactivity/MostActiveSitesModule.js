

Wikijump.modules.MostActiveSitesModule = {};

Wikijump.modules.MostActiveSitesModule.vars = {}

Wikijump.modules.MostActiveSitesModule.listeners = {
	changeTime: function(e, range){
		if(!Wikijump.modules.MostActiveSitesModule.vars['box']){
			var box = YAHOO.util.Event.getTarget(e);
			do{
				box = box.parentNode;
			}while(box && box.className != 'most-active-sites-box')
			Wikijump.modules.MostActiveSitesModule.vars['box'] = box;
		}

		OZONE.ajax.requestModule('wiki/sitesactivity/MostActiveSitesModule', {range: range}, Wikijump.modules.MostActiveSitesModule.callbacks.changeTime);

	}

}

Wikijump.modules.MostActiveSitesModule.callbacks = {
	changeTime: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		Wikijump.modules.MostActiveSitesModule.vars['box'].innerHTML = r.body.replace(/<div[^>]>/, '').replace(/<\/div>\s*$/, '');
	}
}
