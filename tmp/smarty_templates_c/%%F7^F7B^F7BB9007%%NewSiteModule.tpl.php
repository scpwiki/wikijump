<?php /* Smarty version 2.6.7, created on 2008-12-06 17:49:06
         compiled from /var/www/wikidot/templates/modules/newsite/NewSiteModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/newsite/NewSiteModule.tpl', 6, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/newsite/NewSiteModule.tpl', 43, false),)), $this); ?>
<div id="new-site-box">
	<?php if ($this->_tpl_vars['notLogged']): ?>
		<h3>We are almost ready to create a new site for you<?php if ($this->_tpl_vars['unixName']): ?> at <span style="text-decoration: underline"><?php echo $this->_tpl_vars['unixName']; ?>
.<?php echo $this->_tpl_vars['URL_DOMAIN']; ?>
</span><?php endif; ?>...</h3>

		<p>
			<?php $this->_tag_stack[] = array('t', array('1' => $this->_tpl_vars['SERVICE_NAME'])); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>However you would need to have a valid user account at %1 so that we could identify you in the future.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</p>
		<table style="margin: 1em auto">
			<tr>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;" onclick="WIKIDOT.page.listeners.loginClick(event)"
							><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Log in<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
					</div>
					<p>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>if you already have a Wikidot account<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
					</p>
				</td>
				<td style="padding: 1em; font-size: 140%">
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>or<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
				</td>
				<td style="text-align: center; padding: 1em">
					<div style="font-size: 180%; font-weight: bold;">
						<a href="javascript:;"  onclick="WIKIREQUEST.createAccountSkipCongrats=true;WIKIDOT.page.listeners.createAccount(event)"
							><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Get a new account<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
					</div>
				</td>
			</tr>
		</table>

	<?php else: ?>


		<div class="error-block" id="new-site-form-errors" style="display: none"></div>

		<form id="new-site-form">
			<table class="form">
				<tr>
					<td>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Site name<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
					</td>
					<td>
						<input class="text" type="text" id="new-site-name" name="name" size="30" value="<?php echo ((is_array($_tmp=$this->_tpl_vars['siteName'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
" />
						<div class="sub">
							<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Appears on the top-left corner of your Wikidot site.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
						</div>
					</td>
				</tr>
				<tr>
					<td>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Tagline<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
					</td>
					<td>
						<input class="text" type="text" name="subtitle" size="30" />
						<div class="sub">
							<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Appears beneath the name.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
						</div>
					</td>
				</tr>
				<tr>
					<td>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Web address<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
					</td>
					<td>
						<input class="text" type="text" id="new-site-unixname" name="unixname" size="20" style="text-align: right" value="<?php echo ((is_array($_tmp=$this->_tpl_vars['unixName'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
"/>.<?php echo $this->_tpl_vars['URL_DOMAIN']; ?>

						<div class="sub">
							<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Only alphanumeric [a-z0-9] and "-" (dash) characters allowed.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
						</div>
					</td>
				</tr>
								<tr>
					<td>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Initial template<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
					</td>
					<td>
						<select name="template">
							<option value=""><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>-- please select initial layout for your wiki --<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></option>
							<?php if (count($_from = (array)$this->_tpl_vars['templates'])):
    foreach ($_from as $this->_tpl_vars['template']):
?>
								<option value="<?php echo $this->_tpl_vars['template']->getSiteId(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['template']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</option>
							<?php endforeach; endif; unset($_from); ?>
						</select>
					</td>
				<tr>
					<td>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Private site?<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
					</td>
					<td>
						<input type="checkbox" name="private" class="checkbox">
						<div class="sub">
							<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>If you check this, the site is visible only to its members.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
						</div>
					</td>

				</tr>
							<tr>
					<td>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Please confirm:<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
					</td>
					<td>
						<input type="checkbox" name="tos" class="checkbox">
						<br/>
						<?php $this->_tag_stack[] = array('t', array('1' => $this->_tpl_vars['URL_HOST'])); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>I have read and agree to the <a href="http://%1/legal:terms-of-service"
						target="_blank">Terms of Service</a>.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>

					</td>

				</tr>
			</table>
			<div class="buttons">
				<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Get a new wiki<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.NewSiteModule.listeners.next3(event)"/>
			</div>
		</form>



	<?php endif; ?>
</div>