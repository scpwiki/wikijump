WIKIDOT.ManageUsersModule = {
		
	'save': function(form) {

		var params = OZONE.utils.formToArray(form);
		params.action = 'ManageUsersAction';
		params.event = 'save';

		OZONE.ajax.requestModule(null, params, function(r) {
			document.location = document.location;
		});
		
	}
};
