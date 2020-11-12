

Wikijump.modules.MostActiveForumsModule = {};

Wikijump.modules.MostActiveForumsModule.vars = {}

Wikijump.modules.MostActiveForumsModule.listeners = {
	changeTime: function(e, range){
		if(!Wikijump.modules.MostActiveForumsModule.vars['box']){
			var box = YAHOO.util.Event.getTarget(e);
			do{
				box = box.parentNode;
			}while(box && box.className != 'most-active-forums-box')
			Wikijump.modules.MostActiveForumsModule.vars['box'] = box;
		}

		OZONE.ajax.requestModule('wiki/sitesactivity/MostActiveForumsModule', {range: range}, Wikijump.modules.MostActiveForumsModule.callbacks.changeTime);

	}

}

Wikijump.modules.MostActiveForumsModule.callbacks = {
	changeTime: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		Wikijump.modules.MostActiveForumsModule.vars['box'].innerHTML = r.body.replace(/<div[^>]*>/, '').replace(/<\/div>\s*$/, '');
	}
}
