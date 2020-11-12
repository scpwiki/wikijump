

Wikijump.modules.UserInfoModule = {};

Wikijump.modules.UserInfoModule.vars = {};

Wikijump.modules.UserInfoModule.listeners = {
  	tabClick: function(e){
  		// delete "active" state from all other elements
  		var div = document.getElementById("user-info-side");
  		var as = div.getElementsByTagName('a');
  		for(var i=0; i<as.length; i++){
  			if(as[i].className == 'active'){
  				as[i].className = '';
  			}

  		}
  		this.className="active";
  		var moduleName = Wikijump.modules.UserInfoModule.vars.modulesMapping[this.id];
  		var parms = new Object();
		parms['user_id'] = USERINFO.userId;
		OZONE.ajax.requestModule(moduleName, parms, Wikijump.modules.UserInfoModule.callbacks.tabClick);
  	},

  	flagUser: function(e, userId){
		OZONE.ajax.requestModule('report/FlagUserModule', {targetUserId: userId}, Wikijump.modules.UserInfoModule.callbacks.flagUser);

	},

	addContact: function(e, userId){
		OZONE.ajax.requestModule('userinfo/UserAddToContactsModule', {userId: userId}, Wikijump.modules.UserInfoModule.callbacks.addContact);
	}

}

Wikijump.modules.UserInfoModule.callbacks = {
	tabClick: function(response){
		if(!Wikijump.utils.handleError(response)) {return;}
		// experimental fade-out ;-)
		OZONE.utils.setInnerHTMLContent("user-info-area", response.body);
		OZONE.utils.formatDates("user-info-area");
	},
	flagUser: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}	,

	addContact: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}

}

Wikijump.modules.UserInfoModule.init = function(){
	var tabIds = ['ui-profile-b', 'ui-member-b', 'ui-admin-b', 'ui-contrib-b', 'ui-posts-b', 'ui-moderator-b'];
	YAHOO.util.Event.addListener(tabIds, "click", Wikijump.modules.UserInfoModule.listeners.tabClick);

	var mm = new Array();
	mm['ui-profile-b'] = "userinfo/UserInfoProfileModule";
	mm['ui-member-b'] = "userinfo/UserInfoMemberOfModule";
	mm['ui-admin-b'] = "userinfo/UserInfoAdminOfModule";
	mm['ui-moderator-b'] = "userinfo/UserInfoModeratorOfModule";
	mm['ui-contrib-b'] = "userinfo/UserChangesModule";
	mm['ui-posts-b'] = "userinfo/UserRecentPostsModule";
	// etc...
	Wikijump.modules.UserInfoModule.vars.modulesMapping = mm;

}

Wikijump.modules.UserInfoModule.init();
