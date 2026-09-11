%global vigil_stage %{_sourcedir}

%global __os_install_post %{nil}
%global debug_package %{nil}

Name:           vigil
Version:        %{vigil_version}
Release:        1%{?dist}
Summary:        Host protection agent — watches the host it runs on
License:        Apache-2.0
Vendor:         RSQA
AutoReqProv:    no
Packager:       RSQA <vigil@rsqa.space>

%description
vigil reads the host it is installed on — listening sockets, accounts and
logins, and the files that decide who may enter — compares each reading with the
previous one, and turns the difference into findings.

This package installs three programs: vigild, the daemon systemd runs, vigil,
the terminal console that reads its local socket, and vigil-audit-plugin, which
auditd starts to hand this host's program launches over.

%prep

%build

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a %{vigil_stage}/. %{buildroot}/

%files
%attr(0755,root,root) /usr/sbin/vigild
%attr(0755,root,root) /usr/bin/vigil
%attr(0755,root,root) /usr/sbin/vigil-audit-plugin
%dir %attr(0700,root,root) /etc/vigil
%config(noreplace) %attr(0600,root,root) /etc/vigil/vigil.yaml
%config(noreplace) %attr(0644,root,root) /etc/logrotate.d/vigil
%dir %attr(0750,root,root) /etc/audit
%dir %attr(0750,root,root) /etc/audit/rules.d
%dir %attr(0750,root,root) /etc/audit/plugins.d
%config(noreplace) %attr(0640,root,root) /etc/audit/rules.d/vigil-exec.rules
%config(noreplace) %attr(0640,root,root) /etc/audit/plugins.d/vigil.conf
%attr(0644,root,root) /usr/lib/systemd/system/vigild.service
%attr(0644,root,root) /usr/lib/systemd/system/vigil-firewall.service
%attr(0644,root,root) /usr/lib/systemd/system/vigil-firewall.timer
%attr(0644,root,root) /usr/lib/tmpfiles.d/vigil.conf
%dir %attr(0700,root,root) /var/lib/vigil
%dir %attr(0700,root,root) /var/lib/vigil/firewall
%dir %attr(0700,root,root) /var/log/vigil
%dir %attr(0755,root,root) /usr/share/doc/vigil
%doc %attr(0644,root,root) /usr/share/doc/vigil/README.md
%doc %attr(0644,root,root) /usr/share/doc/vigil/NOTICE
%doc %attr(0644,root,root) /usr/share/doc/vigil/THIRD-PARTY.md
%license %attr(0644,root,root) /usr/share/doc/vigil/copyright

%post
if [ -x /usr/bin/systemd-tmpfiles ]; then
    /usr/bin/systemd-tmpfiles --create /usr/lib/tmpfiles.d/vigil.conf >/dev/null 2>&1 || true
fi
for directory in /run/vigil /var/lib/vigil /var/lib/vigil/firewall /var/log/vigil; do
    [ -d "$directory" ] || mkdir -p "$directory"
    chmod 0700 "$directory"
done

if [ -d /run/systemd/system ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true
    if [ "$1" -ge 2 ]; then
        systemctl try-restart vigild.service >/dev/null 2>&1 || true
    fi
fi

if [ -d /etc/audit ]; then
cat <<'AUDIT'

vigil watches program launches through auditd, and auditd has to be told twice:

  augenrules --load          loads /etc/audit/rules.d/vigil-exec.rules
  systemctl restart auditd   starts the plugin in /etc/audit/plugins.d/vigil.conf

This package does neither. Until both are done vigil reads /var/log/audit/audit.log instead, a
reading late and exposed to rotation, and its own health says which of the two is missing.

AUDIT
fi

if [ "$1" -eq 1 ]; then
cat <<'NOTICE'

vigil is installed and is not running yet.

  1. vigild configure --dry-run  — ask THIS host what it can watch, and print the
     configuration for it without writing anything. `vigild configure --force` then
     writes it to /etc/vigil/vigil.yaml, keeping the current file as .previous;
  2. read /etc/vigil/vigil.yaml — every value in it is already the default, so the
     file this package installed is a working configuration as it stands;
  3. systemctl enable --now vigild
  4. vigil ui                    — the console, once the daemon is up

Reading the nftables ruleset is done by vigil-firewall.timer, which vigild.service pulls in:
/usr/sbin/nft runs there and not inside the agent, so the agent keeps the two capabilities it
had. To stop that reading and keep everything else:  systemctl mask vigil-firewall.timer

NOTICE
# A first installation writes the line and starts the timer; an upgrade never touches the
# administrator's file, for the same reason this package does not load audit rules or restart
# auditd on a running host. Neither outcome may fail the installation.
if /usr/sbin/vigild collector firewall enable > /tmp/vigil-firewall-enable.$$ 2>&1; then
    sed 's/^/  /' /tmp/vigil-firewall-enable.$$
else
    echo "vigil: the firewall collector was left switched off on this host:"
    sed 's/^/  /' /tmp/vigil-firewall-enable.$$
    echo "vigil: switch it on when that is fixed:  vigild collector firewall enable"
fi
rm -f /tmp/vigil-firewall-enable.$$
else
echo
echo "vigil: this is an upgrade, so /etc/vigil/vigil.yaml was not touched. To switch"
echo "vigil: the firewall collector on:  vigild collector firewall enable"
echo
fi

%preun
if [ "$1" -eq 0 ] && [ -d /run/systemd/system ]; then
    systemctl --no-reload disable vigild.service vigil-firewall.timer >/dev/null 2>&1 || true
    systemctl stop vigild.service vigil-firewall.timer >/dev/null 2>&1 || true
fi

%postun
if [ -d /run/systemd/system ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true
fi

if [ "$1" -eq 0 ]; then
    if command -v auditctl >/dev/null 2>&1; then
        echo "vigil: the vigil_exec audit rule stays loaded in the kernel until the rules are"
        echo "vigil: reloaded:  augenrules --load  (and restart auditd to drop the plugin)."
    fi

    kept=""
    [ -d /var/lib/vigil ] && kept="$kept /var/lib/vigil"
    [ -d /var/log/vigil ] && kept="$kept /var/log/vigil"
    if [ -n "$kept" ]; then
        echo "vigil: kept on purpose:$kept"
        echo "vigil: findings are the record of what happened on this host."
        echo "vigil: remove them deliberately if you want them gone:  rm -rf$kept"
    fi
fi

%changelog
