/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.modules.PageRateWidgetModule = {};
WIKIDOT.modules.PageRateWidgetModule.vars={};

WIKIDOT.modules.PageRateWidgetModule.listeners = {
	rate: function(e, points, force){
		if(points != 1 && points != -1){ return;}
		
		var p = new Object();
		p.action = "RateAction";
		p.event = "ratePage";
		p.points = points;
		if(force){
			p.force = "yes";
		}
		p.pageId = WIKIREQUEST.info.pageId;
		
		WIKIDOT.modules.PageRateWidgetModule.vars.points = points;

		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PageRateWidgetModule.callbacks.rate);
		
	},
	
	cancelVote: function(e){
		var p = new Object();
		p.action = "RateAction";
		p.event = "cancelVote";
		p.pageId = WIKIREQUEST.info.pageId;
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PageRateWidgetModule.callbacks.rate);
	}
}

WIKIDOT.modules.PageRateWidgetModule.callbacks = {
	rate: function(r){
		
		if(r.status == 'already_voted'){
			var w = new OZONE.dialogs.Dialog();
			w.content = r.body;
			w.show();
			return;
		}
		
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.dialog.cleanAll();
		var el = $("prw54353");
		if(el){
			var nu = el.innerHTML;
			nu = nu.replace(/\+/,'');
			nu = nu*1  + 1*r.points;
			if(nu>0){
				nu = '+'+nu;
			}
			
			var eff = new fx.Opacity(el, {duration: 200});
			eff.setOpacity(0);
			el.innerHTML = nu;
			eff.custom(0,1);
		}
		var el = $("prw54354");
		if(el){
			var nu = el.innerHTML;
			nu = nu.replace(/\+/,'');
			nu = nu*1  +  1*r.points;
			if(nu>0){
				nu = '+'+nu;
			}
			
			var eff = new fx.Opacity(el, {duration: 200});
			eff.setOpacity(0);
			el.innerHTML = nu;
			eff.custom(0,1);
		}
		var el = $("prw54355");
		if(el){
			var nu = el.innerHTML;
			nu = nu.replace(/\+/,'');
			nu = nu*1  +  1*r.points;
			if(nu>0){
				nu = '+'+nu;
			}
			
			var eff = new fx.Opacity(el, {duration: 200});
			eff.setOpacity(0);
			el.innerHTML = nu;
			eff.custom(0,1);
		}
	
	}	
}

WIKIDOT.modules.PageRateWidgetModule.init = function(){
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

WIKIDOT.modules.PageRateWidgetModule.init();
