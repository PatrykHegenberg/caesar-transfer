import 'dart:io';

import 'package:caesar_transfer/messages/ressource.pb.dart';
import '../messages/generated.dart';
import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:shared_preferences/shared_preferences.dart';

const backColor = Color(0xFF32363E);
const highlightColor = Color(0xFF98C379);
const textColor = Color(0xFFABB2BF);

class ReceiveScreen extends StatefulWidget {
  @override
  _ReceiveScreenState createState() => _ReceiveScreenState();
}

class _ReceiveScreenState extends State<ReceiveScreen> {
  String appOrigin = '';
  final myController = TextEditingController();
  String greetingText = '';
  bool _showScanner = false;

  Widget _buildQRScanner() {
    if (Platform.isIOS || Platform.isAndroid) {
      return MobileScanner(
        controller: MobileScannerController(
            detectionSpeed: DetectionSpeed.noDuplicates),
        onDetect: (barcode) {
          if (barcode.raw == null) {
            debugPrint('Failed to scan qr code');
          } else {
            final String code = barcode.raw.toString();
            setState(() {
              greetingText = code;
              _showScanner = false;
            });
          }
        },
      );
    } else {
      _showScanner = false;
      return const SizedBox.shrink();
    }
  }

  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs'; // Laden Sie die app_origin
  }

  void _sendTransferNameToRust(String appOrigin) {
    final String transferName =
        (greetingText.isEmpty) ? myController.text.trim() : greetingText;
    if (transferName.isNotEmpty) {
      TransferName(relay: appOrigin, transferName: transferName)
          .sendSignalToRust();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: backColor,
      body: Center(
          child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          if (!_showScanner)
            GestureDetector(
              onTap: () {
                setState(() {
                  _showScanner = true;
                });
              },
              child: Container(
                width: 200,
                height: 200,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: textColor,
                ),
                child: const Center(
                  child: Icon(
                    Icons.qr_code,
                    color: highlightColor,
                    size: 100,
                  ),
                ),
              ),
            ),
          if (_showScanner)
            Container(
              width: MediaQuery.of(context).size.width * 0.8,
              height: MediaQuery.of(context).size.height * 0.5,
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(16),
              ),
              child: _buildQRScanner(),
            ),
          const SizedBox(height: 32),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: SizedBox(
              width: MediaQuery.of(context).size.width * 0.5,
              child: TextField(
                controller: myController,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: highlightColor,
                ),
                decoration: const InputDecoration(
                  labelText: 'Enter Transfername',
                  labelStyle: TextStyle(color: Colors.white54),
                  enabledBorder: UnderlineInputBorder(
                    borderSide: BorderSide(color: Colors.white),
                  ),
                  focusedBorder: UnderlineInputBorder(
                    borderSide: BorderSide(color: Colors.white),
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: textColor,
              foregroundColor: highlightColor,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(20),
              ),
            ),
            onPressed: () {
              loadSettings().then((_) => _sendTransferNameToRust(appOrigin));
            },
            child: const Text('Receive'),
          ),
          Text(
            greetingText,
            style: const TextStyle(color: Colors.white),
          ),
        ],
      )),
    );
  }
}
