<div class="owindow" style="width: 55em;">
	<div class="title">
		{t}Secure login{/t}
	</div>	
	<div class="content">
		<table style="width: 95%;" >
			<tr>
				<td style="width: 50%; text-align: center;  padding: 10px;">
					
					
					<iframe src="/common--misc/blank.html"
						id="login-iframe"
						scrolling="no" frameBorder="0"></iframe>
					
					<div style="font-size: 90%; text-align: center;">
						<a href="javascript:;" onclick="WIKIDOT.page.listeners.passwordRecoveryClick(event)">{t}forgotten password?{/t}</a>
						|
						<a href="javascript:;" onclick="WIKIDOT.page.listeners.createAccount(event)">{t}no account yet?{/t}</a>
					</div>
				</td>
				<td style="padding: 10px; border-left: 1px solid #999">
					<p>
						{t escape=no}This site is a part of the <a href="http://{$URL_HOST}">{$SERVICE_NAME}</a>
						and uses shared user authentication.{/t}
					</p>
					<p>
						{t}<b>Problems with signing in?</b> Try to uncheck the <em>bind to IP</em> option.{/t}
					</p>
					<p style="text-align: center; font-weight: bold">
						{t}This login form is secure.{/t} <span id="secure-login-info">[?]</span>
					</p>
					
						
		 		</td>
		 	</tr>
		</table>
		<div id="secure-login-info-hovertip" style="display: none">
			{t}Your login data (email and password) is transferred in an encrypted form (RSA 512-byte key) to prevent unauthorized access
			or identity theft.{/t}
			<br/>
			{t}Anyway - it is quite impossible for anyone not authorized to intercept your password.{/t}
		</div>
		
	</div>

	
</div>
