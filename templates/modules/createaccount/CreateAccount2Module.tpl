{loadmacro set="Forms"}

<div class="owindow" style="width: 50em">
	<form id="reg2" action="dummy" method="post" onkeypress="return OZONE.utils.disableEnterKey(event)">
	
	<div class="title">
			{t}Create account: email verification{/t}
		</div>
		<div class="content">
			<h1>{t}Verify your email{/t}: <span id="ca-email"></span></h1>
			<p>
				{$name|escape}, {t 1=$email}please check your email account <b>%1</b>. In order to protect from spamming and avoid 
				communication issues in the future we need to check if the email
				address you have provided is valid. In your mailbox you can (most likely) find an email
				with a code that should be entered in the box below.{/t} 
			</p>
			<p>
				{t}Please do not leave this page or your registration process will be interrupted.{/t}
			</p>
			
			<div class="error-block" id="ca-error-block" style="display: none"></div>
			<table class="form">
				<tr>
					<td>
						{t}Verification code{/t}:
					</td>
					<td>
						<input class="text" type="text" name="evercode" id="ca-evercode" size="20"/>
					</td>
				</tr>
			</table>
			
			<p>
				{t}If you have not received your code for more than 5 minutes it could indicate one of the following problems{/t}:
			</p>
			<ul>
				<li>{t}you have misspelled the email address - simply go back and correct it{/t};<li>
				<li>{t}your mail account does not accept emails - is it "full"? is your mail
					server all right? - check your email account{/t};</li>
				<li>{t 1=$SERVICE_NAME}%1 servers are unable to send the message - try again later and report
					a problem if we fail again{/t}.</li>
			</ul>
			
			{*<p>
				{$evcode}
			</p>*}
			
		</div>
		<div class="button-bar">
			<a href="javascript:;" onclick="WIKIDOT.modules.CreateAccountModule.listeners.cancel(event)">{t}cancel{/t}</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.CreateAccount2Module.listeners.backClick(event)">{t}back{/t}</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.CreateAccount2Module.listeners.nextClick(event)">{t}next{/t}</a>
		</div>
	</form>
</div>


