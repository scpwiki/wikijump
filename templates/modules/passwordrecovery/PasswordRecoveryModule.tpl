<div class="owindow" style="width: 50em">
	<div class="title">
		{t}Password recovery{/t}
	</div>
	
	<div class="content">
		<h1>So you have forgotten your password... no problem!</h1>
		
		<p>
			In a few easy steps we will confirm  you are the owner of the user account
			you claim. First please provide the email <u>you have registered with</u>:
		</p>
		
		<div class="error-block" id="recovery-error" style="display: none"></div>
		
		<table class="form">
			<tr>
				<td>
					Your email:
				</td>
				<td>
					<input class="text" name="email" type="text" size="25" id="recovery-email-value"	/>
				</td>
			</tr>
		</table>
		
		<p>In the next steps:</p>
		<ul>
			<li>
				We will send you an email with a verification code. 
				Make sure you have access to the mailbox.
			</li>
			<li>
				You will be asked to enter this code and your new password.
			</li>
		</ul>

		<div class="button-bar">
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">cancel</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.PasswordRecoveryModule.listeners.next1(event);">continue</a>
		</div>
	</form>
	
</div>