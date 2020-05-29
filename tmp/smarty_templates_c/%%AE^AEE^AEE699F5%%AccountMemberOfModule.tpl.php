<?php /* Smarty version 2.6.7, created on 2020-04-09 13:53:15
         compiled from /var/www/wikidot/templates/modules/account/membership/AccountMemberOfModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/account/membership/AccountMemberOfModule.tpl', 1, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/account/membership/AccountMemberOfModule.tpl', 12, false),)), $this); ?>
<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Member of...<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h1>

<?php if ($this->_tpl_vars['memberships']): ?>
	<div class="sites-list">
		<?php if (count($_from = (array)$this->_tpl_vars['memberships'])):
    foreach ($_from as $this->_tpl_vars['member']):
?>
			<div class="site-list-item">
				<?php $this->assign('site', $this->_tpl_vars['member']->getSite()); ?>
				<div class="options">
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>options<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>: <a href="javascript:;" onclick="WIKIDOT.modules.AccountMemberOfModule.listeners.signOff(event, [<?php echo $this->_tpl_vars['site']->getSiteId(); ?>
, '<?php echo $this->_tpl_vars['site']->getName(); ?>
'])"><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>sign off<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
				</div>
				<div class="name">
					<a href="http://<?php echo $this->_tpl_vars['site']->getDomain(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['site']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
				</div>
				<?php if ($this->_tpl_vars['site']->getSubtitle()): ?>
					<div class="subtitle">
						<?php echo ((is_array($_tmp=$this->_tpl_vars['site']->getSubtitle())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>

					</div>
				<?php endif; ?>
				<?php if ($this->_tpl_vars['site']->getDescription()): ?>
					<div class="description">
						<?php echo ((is_array($_tmp=$this->_tpl_vars['site']->getDescription())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>

					</div>
				<?php endif; ?>
				
			</div>
		<?php endforeach; endif; unset($_from); ?>
	</div>
<?php else: ?>
	<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Currently you are not a member of any site :-(<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack);  endif; ?>
<div style="display: none" id="signoff-window">
	<?php $this->_tag_stack[] = array('t', array('escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Are you sure you do not want to be a member of the site <strong>%%SITE_NAME%%</strong> any more?<br/>
	If you have any additional role in this site (admin, moderator) it will be lost too.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
</div>