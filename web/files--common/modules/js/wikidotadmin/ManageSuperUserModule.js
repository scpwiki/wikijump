WIKIDOT.ManageSuperUserModule = {
		
	'save': function(form) {

		var params = OZONE.utils.formToArray(form);
		params.action = 'ManageSuperUserAction';
		params.event = 'save';

		if (params.password1 != params.password2) {
			
			alert("Passwords don't match.");
			
		} else if (params.password1 == '') {
			
			alert("Can't use empty password.");
			
		} else {
		
			OZONE.ajax.requestModule(null, params, function() {
				if (params.key) {
					alert('Saved. You need to log in now.');
					document.location = '/auth:login';
				} else {
					alert('Saved.');
				}
			});
			
		}
		
	}
};