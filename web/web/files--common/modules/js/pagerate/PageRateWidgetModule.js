

Wikijump.modules.PageRateWidgetModule = {};
Wikijump.modules.PageRateWidgetModule.vars={};

Wikijump.modules.PageRateWidgetModule.listeners = {
	rate: function(e, points, force){
		if(points > 5 || points < -1){ return;}

		var p = new Object();
		p.action = "RateAction";
		p.event = "ratePage";
		p.points = points;
		if(force){
			p.force = "yes";
		}
		p.pageId = WIKIREQUEST.info.pageId;

		Wikijump.modules.PageRateWidgetModule.vars.points = points;

		OZONE.ajax.requestModule(null, p, Wikijump.modules.PageRateWidgetModule.callbacks.rate);

	},

	cancelVote: function(e){
		var p = new Object();
		p.action = "RateAction";
		p.event = "cancelVote";
		p.pageId = WIKIREQUEST.info.pageId;

		OZONE.ajax.requestModule(null, p, Wikijump.modules.PageRateWidgetModule.callbacks.rate);
	}
}

Wikijump.modules.PageRateWidgetModule.callbacks = {
	rate: function(r){

		if(r.status == 'already_voted'){
			var w = new OZONE.dialogs.Dialog();
			w.content = r.body;
			w.show();
			return;
		}

		if(!Wikijump.utils.handleError(r)) {return;}
		OZONE.dialog.cleanAll();
		var el = $("prw54353");
		if(r.type != "S") {
			if (el) {
				var nu = el.innerHTML;
				nu = nu.replace(/\+/, '');
				nu = nu * 1 + 1 * r.points;
				if (nu > 0) {
					nu = '+' + nu;
				}
			}
		}
		else {
			var nu = parseFloat(el.innerHTML);
			var pts = parseFloat(r.points);
			var votes = parseFloat(r.votes);
			if(pts < 0) { // Changed vote downward or removed entirely, we can't see which from here.
				nu = (((votes + 1) * nu) + pts) / (votes);
			}
			else {
				nu = ((votes * nu) + pts) / (votes + 1);
			}

		}
			var eff = new fx.Opacity(el, {duration: 200});
			eff.setOpacity(0);
			el.innerHTML = nu;
			eff.custom(0,1);
		var el = $("prw54354");
		if(r.type != "S") {
			if (el) {
				var nu = el.innerHTML;
				nu = nu.replace(/\+/, '');
				nu = nu * 1 + 1 * r.points;
				if (nu > 0) {
					nu = '+' + nu;
				}
			}
		}
		else {
			if(el) {
				var nu = el.innerHTML;
				var pts = r.points;
				var votes = r.votes;
				nu = ((votes * nu) + pts) / (votes + 1);
			}
		}

			var eff = new fx.Opacity(el, {duration: 200});
			eff.setOpacity(0);
			el.innerHTML = nu;
			eff.custom(0,1);

		var el = $("prw54355");
		if(r.type != "S") {
			if (el) {
				var nu = el.innerHTML;
				nu = nu.replace(/\+/, '');
				nu = nu * 1 + 1 * r.points;
				if (nu > 0) {
					nu = '+' + nu;
				}
			}
		}
		else {
			if(el) {
				var nu = el.innerHTML;
				var pts = r.points;
				nu = nu.replace(/\+/, '');
				nu = reduce((nu, pts) => (nu + pts)) / 2;
			}
		}
			var eff = new fx.Opacity(el, {duration: 200});
			eff.setOpacity(0);
			el.innerHTML = nu;
			eff.custom(0,1);

	}
}

Wikijump.modules.PageRateWidgetModule.init = function(){
	OZONE.dom.onDomReady(function(){
		if($("membership-by-apply-text")){
			// if a rating is already displayed - rather copy the value.
			var el = $("prw54355");
			if(el){
				var nu = el.innerHTML;
				if(nu>0){
					nu = '+'+nu.replace(/\+/,'');;
				}
				$("prw54354").innerHTML = nu;
			}
		}
	}, "dummy-ondomready-block");

}

Wikijump.modules.PageRateWidgetModule.init();
