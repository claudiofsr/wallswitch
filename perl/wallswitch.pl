#!/usr/bin/perl
use strict;                   # see: 'perldoc strict'
use warnings FATAL => 'all';  # see: 'perldoc warnings'
use diagnostics;

use Data::Dumper qw(Dumper);
$Data::Dumper::Terse = 1;
$Data::Dumper::Sortkeys = sub {
    my %h = %{$_[0]};
    # cmp for string comparisons
    [ sort { $h{$b} cmp $h{$a} || $a cmp $b } keys %h ];
};

sub main {
    my $interval = 30 * 60 ; # 30 * 60 segundos = 30 minutos
    my $min_dimension = 800; # minimum dimension h x w
    my $type = qr/\.(?:jpg|jpeg|png)$/i;

    my @dirs = get_all_dir();
    show_msgs(\@dirs, $interval);

    kill_other_instances();

    my %paths;
    get_pictures(\@dirs, $type, \%paths, $min_dimension);

    unless (keys %paths) {
        print "No images found in directories:\n";
        print 'Monitoring directories: '.(Dumper \@dirs)."\n";
        exit;
    }

    my @pictures_path = shuffle_and_show(\%paths);
    my $step = 2; # num items at a time in a Perl foreach loop

    while (1) {
        foreach my $i (0 .. $#pictures_path) {
            last if ($i + $step > $#pictures_path);
            next unless ($i % $step == 0);
            my ($picture1, $picture2) = @pictures_path[$i .. ($i + $step)];
            set_wallpaper($picture1, $picture2, $min_dimension);
            sleep $interval;
        }
    }
}

sub get_all_dir {
    my $dir1 = "$ENV{HOME}/Pictures";     # put your wallpaper folder here
    my $dir2 = "$ENV{HOME}/Imagens";      # put your wallpaper folder here
    my $dir3 = "/usr/share/wallpapers";   # kde wallpapers location
    my $dir4 = "/usr/share/backgrounds";  # Gnome wallpapers location
    my $dir5 = "/usr/share/antergos/wallpapers";
    my $dir6 = "/tmp/teste";              # testar diretório inexistente

    my @all_dir = ($dir1, $dir2, $dir3, $dir4, $dir5, $dir6);
    # print 'All directories: '.(Dumper \@all_dir)."\n";

    return @all_dir;
}

sub show_msgs {
    my ($dirs, $interval) = @_;

    my $prog_name  = "wallswitch.pl (random background image) exibe papeis de parede de forma aleatória.";
    my $descr1     = "Dependências: imagemagick (image viewing/manipulation program) e feh (fast and light imlib2-based image viewer).";
    my $descr2     = "Intervalo entre cada papel de parede: $interval segundos.";
    my $Author     = 'Claudio Fernandes de Souza Rodrigues (claudiofsrodrigues@gmail.com)';
    my $date       = '18 de Julho de 2024 (início: 12 de Junho de 2019)';
    my $version    = '0.50';

    print "\n $prog_name\n $descr1\n $descr2\n $Author\n $date\n versão: $version\n\n";
    print 'Monitoring directories: '.(Dumper $dirs)."\n";
}

sub kill_other_instances {
    my $pid = $$;
    my $pids = encontrar_pid($0);

    foreach my $pid_number ( @$pids ) {
        next if ($pid_number == $pid);
        print "Killing previous instances: kill -9 $pid_number\n";
        system qq{kill -9 $pid_number};
    }
}

sub encontrar_pid {
    my $programa = shift;

    my @pid_a = split /\n|\s/, exec_cmd_system( "pgrep -f $0" );  # script name: $0
    my @pid_b = split /\n|\s/, exec_cmd_system( "pidof -x $programa" );

    my @pid_total = (@pid_a, @pid_b);

    unique( array_ref => \@pid_total );

    print "\@pid_total = (@pid_total)\n\n";

    return \@pid_total;
}

sub exec_cmd_system { # need capture the output
    my $cmd = shift;  # $cmd is a string
    my $output;

    open ( my $fh, '-|', $cmd ) or die "Could not execute <$cmd>: $!\n";
    {
        local $/ = undef; # how can I read an entire file into a string?
        while(<$fh>) { $output = $_; } # $_ is the output of command
    }
    close ($fh);
    chomp ($output);
    return $output;
}

sub unique {
   my %args = ( case       => 'normal',  # ou 'uppercase'
                array_ref  => undef,     # reference to array
                @_,                      # argument pair list goes here
              );

   my $array = $args{array_ref};         # this is a reference of an array
   my %seen;

   my $to = 0;
   for ( my $from = 0; $from < @$array; $from++ ) {
        if ( $args{case} =~ /uppercase/i ) {
         next if ( $seen{ uc($array->[$from]) }++ );
      }
      else {
         next if ( $seen{ $array->[$from] }++ );
      }
      $array->[$to++] = $array->[$from]; # move elements backwards
   }
   splice @$array, $to; # remove tail

   return;
}

sub get_pictures {
    my ($dirs, $type, $paths, $min_dimension) = @_;

    foreach my $dir (@$dirs) {
        OpenDirRecursively($dir, $type, $paths, $min_dimension);
    }
}

sub OpenDirRecursively {
    my ($dir, $type, $paths, $min_dimension) = @_;
    my $seen_file;

    unless (-d "$dir") {
        print "Could not open <$dir> for reading '$!'.\n\n";
        return $seen_file;
    }

    opendir(my $dh, $dir) or die "Could not open directory '$dir': $!";

    while (my $file = readdir($dh)) {
        next if ($file eq '.' or $file eq '..');
        # next if $file =~ /^\.*/;  # Skip hidden files
        my $path = "$dir/$file";

        if (-d $path) {
            # Se $path for um diretório, abrir recursivamente.
            OpenDirRecursively($path, $type, $paths, $min_dimension)
        } else {
            # Do something with the file
            next if ( $file eq 'wallswitch.jpg' );
            next unless ( $file =~ $type ); # We only want this types

            # Verificar se $path contém "backgrounds"
            # Obter informações de imagens somente deste $path
            # Em outros diretórios as imagens foram previamente filtradas/escolhidas
            if ( $path =~ /backgrounds/i ) {
                my $info = get_info($path, $min_dimension);
                # Filtrar figuras com resoluções mínimas
                unless ( $info->{valid} ) {
                    print "excluir path: '$path' ; dimension: $info->{dimension}\n\n";
                    next;
                }
            }

            $paths->{$path}++;
            $seen_file++;
        }
    }

    closedir($dh);
    return $seen_file;
}

sub get_info {
    my ($path, $min_dimension) = @_;

    # Obter 'largura x altura' (width x height) em uma única execução:
    # Exemplo: dimension = 3840x2160
    my $dimension = exec_cmd_system("identify -format '%wx%h' '$path'");

    # Verificar se o comando foi executado com sucesso
    die "Erro ao executar identify: $!" unless defined $dimension;

    # Extrair valores da string usando captura de grupos
    my ($width, $height) = split( /\s*x\s*/, $dimension, -1 );

    # Calcular o mínimo entre largura e altura
    my $min = minimum($width, $height);
    my $valid = $min > $min_dimension; # bool

    my %info = (
        width => $width,
        height => $height,
        min_dimension => $min_dimension,
        path => $path,
        dimension => $dimension,
        valid => $valid,
    );

    print '%info: '.(Dumper \%info)."\n";

    return \%info;
}

sub shuffle_and_show {
    my $paths = shift;
    my @pictures_path = shuffle($paths);
    show_pictures(\@pictures_path);

    return @pictures_path;
}

# Fisher-Yates shuffle
sub shuffle {
    my $hash_ref = shift;

    my @array = keys %$hash_ref;
    my $n_elements = scalar @array;

    foreach my $i ( 0 .. $n_elements - 1 ) {
        my $j = int(rand($n_elements));
        # Swap ith and jth elements
        ($array[$i], $array[$j]) = ($array[$j], $array[$i]);
    }

    return @array;
}

sub show_pictures {
    my $pictures_path = shift;

    my $n_elements = scalar @$pictures_path;
    printf "Foram encontradas %s figuras: (embaralhadas)\n", scalar @$pictures_path;

    foreach my $i ( 0 .. $n_elements - 1 ) {
        printf "\$pictures_path[%3s] = '$pictures_path->[$i]' \n", $i;
    }

    print "\n";
}

sub set_wallpaper {
    my ($picture1, $picture2, $min_dimension) = @_;
    my $desktop = $ENV{DESKTOP_SESSION}; # echo $DESKTOP_SESSION

    if ($desktop =~ /gnome|ubuntu/i) {
        set_gnome_wallpaper($picture1, $picture2, $min_dimension);
    } elsif ($desktop =~ /xfce/i) {
        set_xfce_wallpaper($picture1, $picture2);
    } else { # openbox
        set_openbox_wallpaper($picture1, $picture2);
    }
}

sub set_gnome_wallpaper {
    my ($picture1, $picture2, $min_dimension) = @_;

    my $info1 = get_info($picture1, $min_dimension);
    my ($w1, $h1) = ($info1->{width}, $info1->{height});

    my $info2 = get_info($picture2, $min_dimension);
    my ($w2, $h2) = ($info2->{width}, $info2->{height});

    printf "picture1 = '$picture1' (%s)\n",   $info1->{dimension};
    printf "picture2 = '$picture2' (%s)\n\n", $info2->{dimension};

    my $min_w  = minimum( $w1, $w2 );
    my $min_dw = 2 * $min_w;

    print "w1 = $w1 ; w2 = $w2 ; min_w = $min_w ; min_dw = 2 * $min_w = $min_dw \n";

    my $hresize1 = $h1 * ($min_w/$w1);
    my $hresize2 = $h2 * ($min_w/$w2);
    my $aspect_ratio = $min_w * (2160/3840); # see xrandr: connected primary 3840x2160+0+0

    my $min_h = minimum( $hresize1, $hresize2, $aspect_ratio );

    print "h1 = $h1 ; h2 = $h2 ; hresize1 = $hresize1 ; hresize2 = $hresize2 ;  min_h = minimum( $hresize1, $hresize2, $aspect_ratio ) = $min_h \n\n";

    my $command;

    if ( $w1 > $w2 ) {
        $command = "magick '$picture1'[${min_w}x${hresize1}] '$picture2' ";
    } elsif ( $w1 < $w2 ) {
        $command = "magick '$picture1' '$picture2'[${min_w}x${hresize2}] ";
    } else {
        $command = "magick '$picture1' '$picture2' ";
    }

    $command .= "-gravity Center -crop ${min_dw}x${min_h}+0+0 +append '$ENV{HOME}/wallswitch.jpg' ";

    print "$command\n\n";

    system qq{$command};

    $command = "gsettings set org.gnome.desktop.background";

    # picture-uri or picture-uri-dark

    print "$command picture-options spanned \n";
    print "$command picture-uri-dark '$ENV{HOME}/wallswitch.jpg' \n\n";

    system qq{$command picture-options spanned};
    system qq{$command picture-uri-dark '$ENV{HOME}/wallswitch.jpg'};
}

sub set_xfce_wallpaper {
    my ($picture1, $picture2) = @_;

    my $command;
    $command  = "xfconf-query --channel xfce4-desktop --property /backdrop/screen0/monitorDP-0/workspace0/last-image --set '$picture1' ; ";
    $command .= "xfconf-query --channel xfce4-desktop --property /backdrop/screen0/monitorDP-2/workspace0/last-image --set '$picture2' ";

    print "\$picture1 = '$picture1'\n";
    print "\$picture2 = '$picture2'\n";

    printf "$command \n\n";

    system qq{$command};
}

sub set_openbox_wallpaper {
    my ($picture1, $picture2) = @_;

    my $command = "feh --bg-fill 'picture1' --bg-fill 'picture2' ";

    print "\$picture1 = '$picture1'\n";
    print "\$picture2 = '$picture2'\n";
    printf "$command \n\n";

    system qq{$command};
}

sub maximum {
    my ($max, @vars) = @_;
    for (@vars) {
        $max = $_ if ($_ > $max);
    }
    return $max;
}

sub minimum {
    my ($min, @vars) = @_;
    for (@vars) {
        $min = $_ if ($_ < $min);
    }
    return $min;
}

main();
