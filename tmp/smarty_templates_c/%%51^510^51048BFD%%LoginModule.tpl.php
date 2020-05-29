<?php /* Smarty version 2.6.7, created on 2008-12-06 17:18:25
         compiled from /var/www/wikidot/templates/modules/login/LoginModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/login/LoginModule.tpl', 7, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/login/LoginModule.tpl', 7, false),)), $this); ?>

<div class="error-block" id="loginerror" style="display: none"></div>
<form id="login-form" action="common--html/dummy.html" method="post"
	onsubmit="WIKIDOT.modules.LoginModule.listeners.loginClick(event)">		
	
	<?php if ($this->_tpl_vars['user']): ?>
		<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Hello<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>, <span style="font-size:130%; font-weight: bold"><?php echo ((is_array($_tmp=$this->_tpl_vars['user']->getNickName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</span>
		<br/>
		<br/>
	<?php else: ?>			
		<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Email<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		<br/>
		<input class="text" name="name" type="text" size="25" id="login-form-name"/>
		<br/><br/>
	<?php endif; ?>
	<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Password<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
	<br/>
	<input class="text" name="password"  type="password" size="25" id="login-form-password"/>
	<br/><br/>
	<?php if ($this->_tpl_vars['user']): ?>
		Not <?php echo ((is_array($_tmp=$this->_tpl_vars['user']->getNickName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
?<br/>
		<a href="javascript:;" style="font-size: 85%" onclick="WIKIDOT.modules.LoginModule.listeners.switchUser(event)"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Log in as a different user<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>.
		<br/><br/>
	<?php endif; ?>
	
	<input class="checkbox" name="keepLogged" type="checkbox" 
		id="login-form-keeplogged" checked="checked"/>
	<label for="login-form-keeplogged"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Keep me logged in unless I log out<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></label> <span id="keep-logged-info">[?]</span>
	<br/>
	&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;(Uncheck if on a shared computer)
	
	<br/>
	<input class="checkbox" name="bindIP" type="checkbox" checked="checked"
		id="login-form-bindip"/>
	<label for="login-form-bindip"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Bind session to my IP<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></label> <span id="bind-ip-info">[?]</span>
	

	<hr/>
	<p>
		<a href="javascript:;" onclick="WIKIDOT.page.listeners.passwordRecoveryClick(event)"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Forgot your password?<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
	</p>
	<hr/>
	<p>
		<a href="/auth:newaccount"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>No account yet? Get one!<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
	</p>
	<div class="buttons" >
		<input type="button" onclick="WIKIDOT.modules.LoginModule.listeners.cancel(event)" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>cancel<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>"/>
		<input type="submit" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>login<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" style="font-weight: bold"/>
	</div>
	<div id="keep-logged-info-hovertip">
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Select this option and you will not be automatically logged-out after 30 minutes 
			of inactivity.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> 
		</div>
		<div id="bind-ip-info-hovertip">
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>In order to increase security it is advised to bind the session to the IP address of the computer you use.
			Problems? When your computer changes the IP number (via DHCP) you might loose your session
			(and log out) occasionally. Option recommended.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</div>
</form>
				