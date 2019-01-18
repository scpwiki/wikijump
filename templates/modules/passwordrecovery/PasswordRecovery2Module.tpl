<div class="owindow" style="width: 50em">
	<div class="title">
		Password recovery
	</div>
	
	<div class="content">
		<h1>Email verification</h1>
		
		<p>
			Now check your email address ({$email|escape}) for the letter from {$SERVICE_NAME}. Copy the
			verification code from the letter into the form below. Please also set a new password.
		</p>
		
		<div class="error-block" id="recovery-error" style="display: none"></div>
		<form id="pr-form">
			<table class="form">
				<tr>
					<td>
						Verification code:
					</td>
					<td>
						<input class="text" type="text" name="evercode" id="pr-evercode" size="15"/>
					</td>
				</tr>
				<tr>
					<td>
						Password:
					</td>
					<td>
						 <input class="text" type="password" name="password" maxlength="30" size="15"/>
						 <div class="sub">
						 	Between 6 and 20 characters.
						 </div>
					</td>
				</tr>
				<tr>
					<td>
						Password (repeat):
					</td>
					<td>
						 <input class="text" type="password" name="password2" maxlength="30" size="15"/>
					</td>
				</tr>
			</table>
		</form>
		<p>
			If you have not received your code for more than 5 minutes it could indicate one of the following problems:
		</p>
		<ul>
			<li>you have misspelled the email address - you can try again;<li>
			<li>your mail account does not accept emails - is it "full"? is your mail
				server all right? - check your email account;</li>
			<li>{$SERVICE_NAME} servers are unable to send the message - try again later and report
				a problem if we fail again.</li>
		</ul>

		<div class="button-bar">
			<a href="javascript:;" onclick="WIKIDOT.modules.PasswordRecoveryModule.listeners.cancel(event)">cancel</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.PasswordRecoveryModule.listeners.next2(event);">continue</a>
		</div>
	</form>
	
</div>