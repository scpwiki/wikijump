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

WIKIDOT.modules.PetitionAdminModule = {};

WIKIDOT.modules.PetitionAdminModule.vars = {
	currentViewTab: 'overview'
}

WIKIDOT.modules.PetitionAdminModule.listeners = {
	newCampaignClick: function(e){
		$('petition-new-campain-box').style.display = 'block';
		OZONE.visuals.scrollTo('petition-new-campain-box');
		$('petition-new-campain-form').reset();
	},
	
	cancelNewCampaignClick: function(e){
		$('petition-new-campain-form').reset();
		$('petition-new-campain-box').style.display = 'none'
		$("petition-new-campaign-error-box").style.display = 'none';
	},
	
	createCampaign: function(e){
		// read data from the form
		var p = OZONE.utils.formToArray($("petition-new-campain-form"));
		p.action = "extra/petition/PetitionAdminAction";
		p.event = "createCampaign";

		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PetitionAdminModule.callbacks.createCampaign);
	},
	viewCampaignClick: function(e, campaignId){
		var p = new Object();
		p.campaignId = campaignId;
		OZONE.ajax.requestModule("extra/petition/admin/ViewPetitionCampaignModule", p, WIKIDOT.modules.PetitionAdminModule.callbacks.viewCampaign);
		setTimeout('OZONE.visuals.scrollTo("petition-admin-module-box");', 300);
	},
	
	viewList: function(e){
		var p = new Object();
		p.withoutBox = true;
		OZONE.ajax.requestModule("extra/petition/admin/PetitionAdminModule", p, WIKIDOT.modules.PetitionAdminModule.callbacks.updateMainBox);
	},
	
	suspendCampaign: function(e, campaignId){
		var p = new Object();
		p.action="extra/petition/PetitionAdminAction";
		p.event="suspendCampaign";
		p.campaignId = campaignId;
		OZONE.ajax.requestModule("extra/petition/admin/ViewPetitionCampaignModule", p, WIKIDOT.modules.PetitionAdminModule.callbacks.updateMainBox);
	},
	resumeCampaign: function(e, campaignId){
		var p = new Object();
		p.action="extra/petition/PetitionAdminAction";
		p.event="resumeCampaign";
		p.campaignId = campaignId;
		OZONE.ajax.requestModule("extra/petition/admin/ViewPetitionCampaignModule", p, WIKIDOT.modules.PetitionAdminModule.callbacks.updateMainBox);
	},
	
	deleteCampaign: function(e, campaignId){
		var con = confirm("Are you absolutely sure you want to delete this campaign and all the " +
				"associated signatures?");
		if(con){
			p = new Object();
			p.action="extra/petition/PetitionAdminAction";
			p.event="deleteCampaign";
			p.campaignId = campaignId;
			OZONE.ajax.requestModule("extra/petition/admin/PetitionAdminModule", p, WIKIDOT.modules.PetitionAdminModule.callbacks.updateMainBox);
			
		}
	},

	browseTabClick: function(e, campaignId){
		$("petition-admin-tab-overview").style.display = "none";
		$("petition-admin-tab-browse").style.display = "block";
		$("petition-admin-tab-download").style.display = "none";
		
		$("petition-admin-view-overview").style.display = "none";
		$("petition-admin-view-browse").style.display = "block";
		$("petition-admin-view-download").style.display = "none";
		
		OZONE.ajax.requestModule("extra/petition/admin/BrowsePetitionSignaturesModule", {campaignId: campaignId},WIKIDOT.modules.PetitionAdminModule.callbacks.browseTabClick); 
		
	},
	downloadTabClick: function(e, campaignId){
		$("petition-admin-tab-overview").style.display = "none";
		$("petition-admin-tab-browse").style.display = "none";
		$("petition-admin-tab-download").style.display = "block";
		
		$("petition-admin-view-overview").style.display = "none";
		$("petition-admin-view-browse").style.display = "none";
		$("petition-admin-view-download").style.display = "block";
	},
	
	saveCollectSettings: function(e, campaignId){
		var p = OZONE.utils.formToArray("petition-collect-form");
		p.action="extra/petition/PetitionAdminAction";
		p.event="saveCollect";
		p.campaignId = campaignId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PetitionAdminModule.callbacks.saveCollectSettings);
		
	},
	
	selectAllSignatures: function(e){
		var chbxs = $("petition-admin-browse-table").getElementsByTagName("input");
		for(var i=0; i<chbxs.length; i++){
			if(chbxs[i].type=="checkbox" && chbxs[i].id.match(/^petition\-signature\-check/)){
				chbxs[i].checked=true;
			}
		}
	},
	deselectAllSignatures: function(e){
		var chbxs = $("petition-admin-browse-table").getElementsByTagName("input");
		for(var i=0; i<chbxs.length; i++){
			if(chbxs[i].type=="checkbox" && chbxs[i].id.match(/^petition\-signature\-check/)){
				chbxs[i].checked=false;
			}
		}
	},
	removeSelectedSignatures:function(e, campaignId){
		var ids = new Array();
		var chbxs = $("petition-admin-browse-table").getElementsByTagName("input");
		for(var i=0; i<chbxs.length; i++){
			if(chbxs[i].type=="checkbox" && chbxs[i].id.match(/^petition\-signature\-check/) && chbxs[i].checked==true){
				ids.push(chbxs[i].id.replace(/^petition\-signature\-check-/, ''));
			}
		}
		
		if(ids.length == 0){
			alert("No signatures have been selected.");
			return;
		}
		
		if(!confirm("Are you sure you want to remove selected "+ids.length+" signature(s)?")){
			return;
		}
		p = new Object();
		p.action="extra/petition/PetitionAdminAction";
		p.event = "removeSignatures";
		p.ids = ids.join(',');
		p.campaignId = campaignId;
		
		OZONE.ajax.requestModule("extra/petition/admin/BrowsePetitionSignaturesModule", p,WIKIDOT.modules.PetitionAdminModule.callbacks.browseTabClick); 
		
	}
		
}

WIKIDOT.modules.PetitionAdminModule.callbacks = {
	createCampaign: function(r){
		if(r.status == "form_error"){
			var ebox = $("petition-new-campaign-error-box");
			ebox.style.display="block";
			ebox.innerHTML = r.message;
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		WIKIDOT.modules.PetitionAdminModule.listeners.viewCampaignClick(null, r.campaignId)
		
	},
	
	updateMainBox: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("petition-admin-module-box").innerHTML = r.body;
		
	},
	
	browseTabClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("petition-admin-view-browse").innerHTML = r.body;
		OZONE.utils.formatDates($("petition-admin-view-browse"));
	},
	saveCollectSettings: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Settings saved";
		w.show();
	},
	
	viewCampaign: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("petition-admin-module-box").innerHTML = r.body;
		
		var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']); 
		myDataSource.scriptQueryParam="q";
		myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";
	
		var myAutoComp = new YAHOO.widget.AutoComplete("petition-land","petition-land-list", myDataSource);
		myAutoComp.formatResult = function(aResultItem, sQuery) { 
			var title = aResultItem[1];
			var unixName = aResultItem[0];
			if(unixName!= null){
				return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 80%;">('+title+')</div>';
			} else {
				return "";
			}
		}
		
		myAutoComp.autoHighlight = false;
		myAutoComp.minQueryLength = 2;
		myAutoComp.queryDelay = 0.5;
	}

}
