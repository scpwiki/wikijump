

Wikijump.modules.ManagerSiteModule = {};

Wikijump.modules.ManagerSiteModule.vars = {
	modulesMapping: new Object(),
	currentId: null

}

Wikijump.modules.ManagerSiteModule.listeners = {
	tabClick: function(e){
		moduleName = Wikijump.modules.ManagerSiteModule.vars.modulesMapping[this.id];
		OZONE.ajax.requestModule(moduleName, null, Wikijump.modules.ManagerSiteModule.callbacks.tabClick);
	},
	clickMenu: function(e){
		var target = YAHOO.util.Event.getTarget(e);
		var id = target.id;
		target = target.parentNode;
		var list = target.getElementsByTagName("ul").item(0);
		if(!list){
			// means this is the link somewhere... at least should be.
			Wikijump.modules.ManagerSiteModule.utils.loadModule(id);
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

Wikijump.modules.ManagerSiteModule.callbacks = {
	menuClick: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		OZONE.utils.setInnerHTMLContent("sm-action-area", r.body);
		OZONE.utils.formatDates("sm-action-area");
		if(r.categories != null){
			Wikijump.modules.ManagerSiteModule.vars.categories = r.categories;
		}

	}

}

Wikijump.modules.ManagerSiteModule.utils = {
	getCategoryById: function(categoryId){
		var categories = Wikijump.modules.ManagerSiteModule.vars.categories;
		for(i=0; i<categories.length; i++){
			if(categories[i]['category_id'] == categoryId){
				return categories[i];
			}
		}
	},
	getCategoryByName: function(name){
		var categories = Wikijump.modules.ManagerSiteModule.vars.categories;
		for(i=0; i<categories.length; i++){
			if(categories[i]['name'] == name){
				return categories[i];
			}
		}
	},

	loadModule: function(id, options){
		var mm = Wikijump.modules.ManagerSiteModule.vars.modulesMapping;
		var module = mm[id];
		if(module){
			// toggle current
			var currentId = Wikijump.modules.ManagerSiteModule.vars.currentId;
			if(currentId) {YAHOO.util.Dom.removeClass(currentId, "active");}
			Wikijump.modules.ManagerSiteModule.vars.currentId = id;
			YAHOO.util.Dom.addClass(id, "active");
			OZONE.ajax.requestModule(module, options, Wikijump.modules.ManagerSiteModule.callbacks.menuClick,
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

Wikijump.modules.ManagerSiteModule.init = function(){
	var tabIds = ["sm-general", "sm-appearance", "sm-license", "sm-permissions",
	"sm-files", "sm-members", "sm-admins", "sm-admins-invite", "sm-navigation",
	"sm-ma", "sm-members-list", "sm-members-invite", "sm-forum-settings", "sm-forum-layout",
	"sm-forum-perm", "sm-forum-perpage",  "sm-templates", "sm-user-blocks", "sm-ip-blocks"];

	var mm = new Object();
	mm['sm-welcome'] = "ManageSite/ManageSiteWelcomeModule";
	mm['sm-general'] = "ManageSite/ManageSiteGeneralModule";
	mm['sm-domain'] = "ManageSite/ManageSiteDomainModule";
	mm['sm-appearance'] = "ManageSite/ManageSiteAppearanceModule";
	mm['sm-customthemes'] = "ManageSite/ManageSiteCustomThemesModule";
	mm['sm-license'] = "ManageSite/ManageSiteLicenseModule";
	mm['sm-permissions'] = "ManageSite/ManageSitePermissionsModule";
	mm['sm-private'] = "ManageSite/ManageSitePrivateSettingsModule";
	mm['sm-members'] = "ManageSite/ManageSiteMembersModule";
	mm['sm-admins'] = "ManageSite/ManageSiteAdminsModule";
	mm['sm-moderators'] = "ManageSite/ManageSiteModeratorsModule";
	mm['sm-admins-invite'] = "ManageSite/ManageSiteAdminsInviteModule";
	mm['sm-files'] = "Files/Manager/FileManagerModule";
	mm['sm-navigation'] = "ManageSite/ManageSiteNavigationModule";
	mm['sm-ma'] = "ManageSite/ManageSiteMembersApplicationsModule";
	mm['sm-members-list'] = "ManageSite/ManageSiteMembersListModule";
	mm['sm-members-invite'] = "ManageSite/ManageSiteMembersInviteModule";
	mm['sm-email-invitations'] = "ManageSite/ManageSiteEmailInvitationsModule";
	mm['sm-invitations-history'] = "ManageSite/ManageSiteInvitationsHistoryModule";
	mm['sm-forum-settings'] = "ManageSite/ManageSiteForumSettingsModule";
	mm['sm-forum-layout'] = "ManageSite/ManageSiteForumLayoutModule";
	mm['sm-forum-perm'] = "ManageSite/ManageSiteForumPermissionsModule";
	mm['sm-templates'] = "ManageSite/ManageSiteTemplatesModule";
	mm['sm-forum-perpage'] = "ManageSite/ManageSitePerPageDiscussionModule";
	mm['sm-forum-recent'] = "ManageSite/ManageSiteForumRecentModule";
	mm['sm-recent-changes'] = "ManageSite/ManageSiteRecentModule";
	mm['sm-user-blocks'] = "ManageSite/Blocks/ManageSiteUserBlocksModule";
	mm['sm-ip-blocks'] = "ManageSite/Blocks/ManageSiteIpBlocksModule";
	mm['sm-pagerate'] = "ManageSite/PageRate/ManageSitePageRateSettingsModule";
	mm['sm-notifications'] = "ManageSite/ManageSiteNotificationsModule";
	mm['sm-ssl'] = "ManageSite/ManageSiteSecureAccessModule";
	mm['sm-users-email-invitations'] = "ManageSite/ManageSiteLetUsersInviteModule";
	mm['sm-renamesite'] = "ManageSite/ManageSiteRenameModule";
	mm['sm-deletesite'] = "ManageSite/ManageSiteDeleteModule";
	mm['sm-email-lists'] = "ManageSite/Elists/ManageSiteEmailListsModule";
	// etc...
	Wikijump.modules.ManagerSiteModule.vars.modulesMapping = mm;

	YAHOO.util.Event.addListener("site-manager-menu", "click", Wikijump.modules.ManagerSiteModule.listeners.clickMenu);

	OZONE.dom.onDomReady(function(){
		var startPage = "sm-welcome";
		if(window.smStartPage){
			startPage = 'sm-' + smStartPage;
		}

		// on DOM complete!!!

		Wikijump.modules.ManagerSiteModule.utils.loadModule(startPage);
	}, "dummy-ondomready-block");

}

Wikijump.modules.ManagerSiteModule.init();
