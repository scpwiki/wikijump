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

WIKIDOT.modules.ManagerSiteModule = {};

WIKIDOT.modules.ManagerSiteModule.vars = {
	modulesMapping: new Object(),
	currentId: null

}

WIKIDOT.modules.ManagerSiteModule.listeners = {
	tabClick: function(e){
		moduleName = WIKIDOT.modules.ManagerSiteModule.vars.modulesMapping[this.id];
		OZONE.ajax.requestModule(moduleName, null, WIKIDOT.modules.ManagerSiteModule.callbacks.tabClick);
	},
	clickMenu: function(e){
		var target = YAHOO.util.Event.getTarget(e);
		var id = target.id;
		target = target.parentNode;
		var list = target.getElementsByTagName("ul").item(0);
		if(!list){
			// means this is the link somewhere... at least should be.
			WIKIDOT.modules.ManagerSiteModule.utils.loadModule(id);
		} else{
			if(target.tagName.toLowerCase() != 'li') {return;}
			// toggle "selected" class
			if(YAHOO.util.Dom.hasClass(target,"selected")){
				var eff = new fx.Opacity(list, {duration: 200});
				eff.custom(1,0);
				tz = target;
				setTimeout('YAHOO.util.Dom.removeClass(tz,"selected")', 200);
				
			}else{
				YAHOO.util.Dom.addClass(target,"selected");
				var eff = new fx.Opacity(list, {duration: 200});
				eff.setOpacity(0);
				eff.custom(0,1);
			}
		}
	}

}

WIKIDOT.modules.ManagerSiteModule.callbacks = {
	menuClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		OZONE.utils.setInnerHTMLContent("sm-action-area", r.body);
		OZONE.utils.formatDates("sm-action-area");
		if(r.categories != null){
			WIKIDOT.modules.ManagerSiteModule.vars.categories = r.categories;
		}

	}	

}

WIKIDOT.modules.ManagerSiteModule.utils = {
	getCategoryById: function(categoryId){
		var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
		for(i=0; i<categories.length; i++){
			if(categories[i]['category_id'] == categoryId){
				return categories[i];
			}
		}	
	},
	getCategoryByName: function(name){
		var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
		for(i=0; i<categories.length; i++){
			if(categories[i]['name'] == name){
				return categories[i];
			}
		}	
	},
	
	loadModule: function(id, options){
		var mm = WIKIDOT.modules.ManagerSiteModule.vars.modulesMapping;
		var module = mm[id];
		if(module){
			// toggle current
			var currentId = WIKIDOT.modules.ManagerSiteModule.vars.currentId;
			if(currentId) {YAHOO.util.Dom.removeClass(currentId, "active");}
			WIKIDOT.modules.ManagerSiteModule.vars.currentId = id;
			YAHOO.util.Dom.addClass(id, "active");
			OZONE.ajax.requestModule(module, options, WIKIDOT.modules.ManagerSiteModule.callbacks.menuClick,
				null, {clearRequestQueue: true});
			
			// make sure the parent is unfolded (if is a list)
			var p = $(id).parentNode.parentNode.parentNode;
			
			var list = p.getElementsByTagName("ul").item(0);
			
			if(list && p.tagName.toLowerCase() == 'li' && !YAHOO.util.Dom.hasClass(p,"selected")){
				// unfold
				YAHOO.util.Dom.addClass(p,"selected");
				var eff = new fx.Opacity(list, {duration: 200});
				eff.setOpacity(0);
				eff.custom(0,1);
			}
		}

	}			

}

WIKIDOT.modules.ManagerSiteModule.init = function(){
	var tabIds = ["sm-general", "sm-appearance", "sm-license", "sm-permissions", 
	"sm-files", "sm-members", "sm-admins", "sm-admins-invite", "sm-navigation", 
	"sm-ma", "sm-members-list", "sm-members-invite", "sm-forum-settings", "sm-forum-layout",
	"sm-forum-perm", "sm-forum-perpage",  "sm-templates", "sm-user-blocks", "sm-ip-blocks"];
	
	var mm = new Object();
	mm['sm-welcome'] = "managesite/ManageSiteWelcomeModule";
	mm['sm-general'] = "managesite/ManageSiteGeneralModule";
	mm['sm-domain'] = "managesite/ManageSiteDomainModule";
	mm['sm-appearance'] = "managesite/ManageSiteAppearanceModule";
	mm['sm-customthemes'] = "managesite/ManageSiteCustomThemesModule";
	mm['sm-license'] = "managesite/ManageSiteLicenseModule";
	mm['sm-permissions'] = "managesite/ManageSitePermissionsModule";
	mm['sm-private'] = "managesite/ManageSitePrivateSettingsModule";
	mm['sm-members'] = "managesite/ManageSiteMembersModule";
	mm['sm-admins'] = "managesite/ManageSiteAdminsModule";
	mm['sm-moderators'] = "managesite/ManageSiteModeratorsModule";
	mm['sm-admins-invite'] = "managesite/ManageSiteAdminsInviteModule";
	mm['sm-files'] = "files/manager/FileManagerModule";
	mm['sm-navigation'] = "managesite/ManageSiteNavigationModule";
	mm['sm-ma'] = "managesite/ManageSiteMembersApplicationsModule";
	mm['sm-members-list'] = "managesite/ManageSiteMembersListModule";
	mm['sm-members-invite'] = "managesite/ManageSiteMembersInviteModule";
	mm['sm-email-invitations'] = "managesite/ManageSiteEmailInvitationsModule";
	mm['sm-invitations-history'] = "managesite/ManageSiteInvitationsHistoryModule";
	mm['sm-forum-settings'] = "managesite/ManageSiteForumSettingsModule";
	mm['sm-forum-layout'] = "managesite/ManageSiteForumLayoutModule";
	mm['sm-forum-perm'] = "managesite/ManageSiteForumPermissionsModule";
	mm['sm-templates'] = "managesite/ManageSiteTemplatesModule";
	mm['sm-forum-perpage'] = "managesite/ManageSitePerPageDiscussionModule";
	mm['sm-forum-recent'] = "managesite/ManageSiteForumRecentModule";
	mm['sm-recent-changes'] = "managesite/ManageSiteRecentModule";
	mm['sm-user-blocks'] = "managesite/blocks/ManageSiteUserBlocksModule";
	mm['sm-ip-blocks'] = "managesite/blocks/ManageSiteIpBlocksModule";
	mm['sm-pagerate'] = "managesite/pagerate/ManageSitePageRateSettingsModule";
	mm['sm-abuse-page'] = "managesite/abuse/ManageSitePageAbuseModule";
	mm['sm-abuse-user'] = "managesite/abuse/ManageSiteUserAbuseModule";
	mm['sm-abuse-anonymous'] = "managesite/abuse/ManageSiteAnonymousAbuseModule";
	mm['sm-notifications'] = "managesite/ManageSiteNotificationsModule";
	mm['sm-backup'] = "managesite/backup/ManageSiteBackupModule";
	mm['sm-ssl'] = "managesite/ManageSiteSecureAccessModule";
	mm['sm-openid'] = "managesite/ManageSiteOpenIDModule";
	mm['sm-users-email-invitations'] = "managesite/ManageSiteLetUsersInviteModule";
	mm['sm-renamesite'] = "managesite/ManageSiteRenameModule";
	mm['sm-deletesite'] = "managesite/ManageSiteDeleteModule";
	mm['sm-email-lists'] = "managesite/elists/ManageSiteEmailListsModule";
	mm['sm-clonesite'] = "managesite/ManageSiteCloneModule";
	// etc...
	WIKIDOT.modules.ManagerSiteModule.vars.modulesMapping = mm;
	
	YAHOO.util.Event.addListener("site-manager-menu", "click", WIKIDOT.modules.ManagerSiteModule.listeners.clickMenu);	
	
	OZONE.dom.onDomReady(function(){	
		var startPage = "sm-welcome";
		if(window.smStartPage){
			startPage = 'sm-' + smStartPage;
		}
		
		// on DOM complete!!!
		
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule(startPage);
	}, "dummy-ondomready-block");
	
}

WIKIDOT.modules.ManagerSiteModule.init();
